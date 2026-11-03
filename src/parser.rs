use std::fmt;
use std::iter::Peekable;

use crate::lexer::{Token, TokenKind};

#[derive(Debug, PartialEq)]
pub enum Node {
    Directive { pseudo_op: String, symbol: String },
    Label(String),
    Instruction { opcode: String, operands: Vec<String> },
}

#[derive(Debug, PartialEq)]
pub struct ParseError {
    /// 1-based line number in the source.
    pub line: usize,
    pub kind: ParseErrorKind,
}

#[derive(Debug, PartialEq)]
pub enum ParseErrorKind {
    MissingSymbol { directive: String },
    /// `found` is `None` at the end of input.
    UnexpectedToken { expected: &'static str, found: Option<TokenKind> },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.kind)
    }
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseErrorKind::MissingSymbol { directive } => {
                write!(f, "expected symbol name after '{}'", directive)
            }
            ParseErrorKind::UnexpectedToken { expected, found: Some(found) } => {
                write!(f, "expected {}, found {}", expected, found)
            }
            ParseErrorKind::UnexpectedToken { expected, found: None } => {
                write!(f, "expected {}, found end of input", expected)
            }
        }
    }
}

impl std::error::Error for ParseError {}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Node>, ParseError> {
    Parser::new(tokens).parse_program()
}

struct Parser {
    tokens: Peekable<std::vec::IntoIter<Token>>,
    /// Line of the most recently consumed token, used for error reporting.
    line: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens: tokens.into_iter().peekable(), line: 1 }
    }

    fn peek(&mut self) -> Option<&TokenKind> {
        self.tokens.peek().map(|token| &token.kind)
    }

    fn next(&mut self) -> Option<TokenKind> {
        let token = self.tokens.next()?;
        self.line = token.line;
        Some(token.kind)
    }

    fn next_if(&mut self, predicate: impl FnOnce(&TokenKind) -> bool) -> Option<TokenKind> {
        let token = self.tokens.next_if(|token| predicate(&token.kind))?;
        self.line = token.line;
        Some(token.kind)
    }

    fn error(&self, kind: ParseErrorKind) -> ParseError {
        ParseError { line: self.line, kind }
    }

    fn at_end_of_statement(&mut self) -> bool {
        matches!(self.peek(), None | Some(TokenKind::Newline))
    }

    fn expect_end_of_statement(&mut self) -> Result<(), ParseError> {
        match self.next() {
            None | Some(TokenKind::Newline) => Ok(()),
            found => Err(self.error(ParseErrorKind::UnexpectedToken { expected: "end of line", found })),
        }
    }

    fn parse_program(&mut self) -> Result<Vec<Node>, ParseError> {
        let mut nodes = Vec::new();
        while let Some(token) = self.next() {
            if token == TokenKind::Newline {
                continue;
            }
            nodes.push(self.parse_statement(token)?);
        }
        Ok(nodes)
    }

    fn parse_statement(&mut self, token: TokenKind) -> Result<Node, ParseError> {
        match token {
            TokenKind::Ident(name) if self.peek() == Some(&TokenKind::Colon) => {
                self.next();
                Ok(Node::Label(name))
            }
            TokenKind::Ident(name) if name.starts_with('.') => self.parse_directive(name),
            TokenKind::Ident(name) => self.parse_instruction(name),
            found => Err(self.error(ParseErrorKind::UnexpectedToken { expected: "statement", found: Some(found) })),
        }
    }

    fn parse_directive(&mut self, pseudo_op: String) -> Result<Node, ParseError> {
        let symbol = match self.next_if(|token| matches!(token, TokenKind::Ident(_))) {
            Some(TokenKind::Ident(symbol)) => symbol,
            _ => String::new(),
        };
        if pseudo_op == ".global" && symbol.is_empty() {
            return Err(self.error(ParseErrorKind::MissingSymbol { directive: pseudo_op }));
        }
        self.expect_end_of_statement()?;
        Ok(Node::Directive { pseudo_op, symbol })
    }

    fn parse_instruction(&mut self, opcode: String) -> Result<Node, ParseError> {
        let mut operands = Vec::new();
        if !self.at_end_of_statement() {
            operands.push(self.parse_operand()?);
            while self.next_if(|token| *token == TokenKind::Comma).is_some() {
                operands.push(self.parse_operand()?);
            }
        }
        self.expect_end_of_statement()?;
        Ok(Node::Instruction { opcode, operands })
    }

    fn parse_operand(&mut self) -> Result<String, ParseError> {
        match self.next() {
            Some(TokenKind::Ident(operand)) | Some(TokenKind::Integer(operand)) => Ok(operand),
            found => Err(self.error(ParseErrorKind::UnexpectedToken { expected: "operand", found })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    fn parse_source(source: &str) -> Result<Vec<Node>, ParseError> {
        parse(tokenize(source).expect("Failed to tokenize"))
    }

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parses_empty_input() {
        assert_eq!(parse_source(""), Ok(vec![]));
    }

    #[test]
    fn parses_directive() {
        assert_eq!(
            parse_source(".global main"),
            Ok(vec![Node::Directive { pseudo_op: ".global".into(), symbol: "main".into() }]),
        );
    }

    #[test]
    fn parses_label() {
        assert_eq!(parse_source("main:"), Ok(vec![Node::Label("main".into())]));
    }

    #[test]
    fn parses_instruction() {
        assert_eq!(
            parse_source("mov rax, 42"),
            Ok(vec![Node::Instruction { opcode: "mov".into(), operands: strings(&["rax", "42"]) }]),
        );
    }

    #[test]
    fn preserves_statement_order() {
        assert_eq!(
            parse_source(".global main\n\nmain:\n    ret\n"),
            Ok(vec![
                Node::Directive { pseudo_op: ".global".into(), symbol: "main".into() },
                Node::Label("main".into()),
                Node::Instruction { opcode: "ret".into(), operands: vec![] },
            ]),
        );
    }

    #[test]
    fn parses_label_and_instruction_on_same_line() {
        assert_eq!(
            parse_source("main: ret"),
            Ok(vec![
                Node::Label("main".into()),
                Node::Instruction { opcode: "ret".into(), operands: vec![] },
            ]),
        );
    }

    #[test]
    fn rejects_global_without_symbol() {
        assert_eq!(
            parse_source(".global"),
            Err(ParseError { line: 1, kind: ParseErrorKind::MissingSymbol { directive: ".global".into() } }),
        );
    }

    #[test]
    fn rejects_missing_operand_after_comma() {
        assert_eq!(
            parse_source("mov rax,"),
            Err(ParseError {
                line: 1,
                kind: ParseErrorKind::UnexpectedToken { expected: "operand", found: Some(TokenKind::Newline) },
            }),
        );
    }

    #[test]
    fn rejects_operands_without_comma() {
        assert_eq!(
            parse_source("mov rax 42"),
            Err(ParseError {
                line: 1,
                kind: ParseErrorKind::UnexpectedToken { expected: "end of line", found: Some(TokenKind::Integer("42".into())) },
            }),
        );
    }

    #[test]
    fn reports_line_of_error() {
        assert_eq!(
            parse_source(".global main\nmain:\n    mov rax 42\n"),
            Err(ParseError {
                line: 3,
                kind: ParseErrorKind::UnexpectedToken { expected: "end of line", found: Some(TokenKind::Integer("42".into())) },
            }),
        );
    }
}
