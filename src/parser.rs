use std::fmt;
use std::iter::Peekable;

use crate::lexer::{Token, TokenKind};

#[derive(Debug, PartialEq)]
pub enum Node {
    Directive { pseudo_op: String, symbol: String },
    Label(String),
    Instruction { mnemonic: Mnemonic, operands: Vec<Operand> },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mnemonic {
    Mov,
    Ret,
}

impl Mnemonic {
    fn from_name(name: &str) -> Option<Self> {
        let mnemonic = match name {
            "mov" => Mnemonic::Mov,
            "ret" => Mnemonic::Ret,
            _ => return None,
        };
        Some(mnemonic)
    }
}

#[derive(Debug, PartialEq)]
pub enum Operand {
    Register(Register),
    Immediate(i64),
    /// A reference to a label or other symbol.
    Symbol(String),
}

/// 64-bit general purpose registers, in the order of their encoding numbers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Register {
    Rax, Rcx, Rdx, Rbx, Rsp, Rbp, Rsi, Rdi,
    R8, R9, R10, R11, R12, R13, R14, R15,
}

impl Register {
    fn from_name(name: &str) -> Option<Self> {
        let register = match name {
            "rax" => Register::Rax,
            "rcx" => Register::Rcx,
            "rdx" => Register::Rdx,
            "rbx" => Register::Rbx,
            "rsp" => Register::Rsp,
            "rbp" => Register::Rbp,
            "rsi" => Register::Rsi,
            "rdi" => Register::Rdi,
            "r8"  => Register::R8,
            "r9"  => Register::R9,
            "r10" => Register::R10,
            "r11" => Register::R11,
            "r12" => Register::R12,
            "r13" => Register::R13,
            "r14" => Register::R14,
            "r15" => Register::R15,
            _ => return None,
        };
        Some(register)
    }
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
    UnknownMnemonic { name: String },
    IntegerOutOfRange { digits: String },
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
            ParseErrorKind::UnknownMnemonic { name } => {
                write!(f, "no such instruction: '{}'", name)
            }
            ParseErrorKind::IntegerOutOfRange { digits } => {
                write!(f, "integer '{}' is out of range", digits)
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

    fn parse_instruction(&mut self, name: String) -> Result<Node, ParseError> {
        let Some(mnemonic) = Mnemonic::from_name(&name) else {
            return Err(self.error(ParseErrorKind::UnknownMnemonic { name }));
        };

        let mut operands = Vec::new();
        if !self.at_end_of_statement() {
            operands.push(self.parse_operand()?);
            while self.next_if(|token| *token == TokenKind::Comma).is_some() {
                operands.push(self.parse_operand()?);
            }
        }
        self.expect_end_of_statement()?;
        Ok(Node::Instruction { mnemonic, operands })
    }

    fn parse_operand(&mut self) -> Result<Operand, ParseError> {
        match self.next() {
            Some(TokenKind::Ident(name)) => match Register::from_name(&name) {
                Some(register) => Ok(Operand::Register(register)),
                None => Ok(Operand::Symbol(name)),
            },
            Some(TokenKind::Integer(digits)) => match digits.parse() {
                Ok(value) => Ok(Operand::Immediate(value)),
                Err(_) => Err(self.error(ParseErrorKind::IntegerOutOfRange { digits })),
            },
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
            Ok(vec![Node::Instruction {
                mnemonic: Mnemonic::Mov,
                operands: vec![Operand::Register(Register::Rax), Operand::Immediate(42)],
            }]),
        );
    }

    #[test]
    fn preserves_statement_order() {
        assert_eq!(
            parse_source(".global main\n\nmain:\n    ret\n"),
            Ok(vec![
                Node::Directive { pseudo_op: ".global".into(), symbol: "main".into() },
                Node::Label("main".into()),
                Node::Instruction { mnemonic: Mnemonic::Ret, operands: vec![] },
            ]),
        );
    }

    #[test]
    fn parses_label_and_instruction_on_same_line() {
        assert_eq!(
            parse_source("main: ret"),
            Ok(vec![
                Node::Label("main".into()),
                Node::Instruction { mnemonic: Mnemonic::Ret, operands: vec![] },
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

    #[test]
    fn parses_symbol_operand() {
        assert_eq!(
            parse_source("mov rax, foo"),
            Ok(vec![Node::Instruction {
                mnemonic: Mnemonic::Mov,
                operands: vec![Operand::Register(Register::Rax), Operand::Symbol("foo".into())],
            }]),
        );
    }

    #[test]
    fn rejects_unknown_mnemonic() {
        assert_eq!(
            parse_source("mvo rax, 1"),
            Err(ParseError { line: 1, kind: ParseErrorKind::UnknownMnemonic { name: "mvo".into() } }),
        );
    }

    #[test]
    fn rejects_integer_out_of_range() {
        assert_eq!(
            parse_source("mov rax, 99999999999999999999"),
            Err(ParseError {
                line: 1,
                kind: ParseErrorKind::IntegerOutOfRange { digits: "99999999999999999999".into() },
            }),
        );
    }
}
