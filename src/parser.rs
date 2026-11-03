use std::fmt;

use crate::lexer::Token;

#[derive(Debug, PartialEq)]
pub enum Node {
    Directive { pseudo_op: String, symbol: String },
    Label(String),
    Instruction { opcode: String, operands: Vec<String> },
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    MissingSymbol { directive: String },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::MissingSymbol { directive } => {
                write!(f, "expected symbol name after '{}'", directive)
            }
        }
    }
}

impl std::error::Error for ParseError {}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Node>, ParseError> {
    Parser::new(tokens).parse_program()
}

struct Parser {
    tokens: std::vec::IntoIter<Token>,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens: tokens.into_iter() }
    }

    fn parse_program(&mut self) -> Result<Vec<Node>, ParseError> {
        let mut nodes = Vec::new();
        while let Some(token) = self.tokens.next() {
            nodes.push(self.parse_statement(token)?);
        }
        Ok(nodes)
    }

    fn parse_statement(&mut self, token: Token) -> Result<Node, ParseError> {
        match token {
            Token::Directive(pseudo_op, symbol) => self.parse_directive(pseudo_op, symbol),
            Token::Label(label) => Ok(Node::Label(label)),
            Token::Instruction(opcode, operands) => Ok(self.parse_instruction(opcode, operands)),
        }
    }

    fn parse_directive(&mut self, pseudo_op: String, symbol: String) -> Result<Node, ParseError> {
        if pseudo_op == ".global" && symbol.is_empty() {
            return Err(ParseError::MissingSymbol { directive: pseudo_op });
        }
        Ok(Node::Directive { pseudo_op, symbol })
    }

    fn parse_instruction(&mut self, opcode: String, operands: Vec<String>) -> Node {
        Node::Instruction { opcode, operands }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parses_empty_input() {
        assert_eq!(parse(vec![]), Ok(vec![]));
    }

    #[test]
    fn parses_directive() {
        let tokens = vec![Token::Directive(".global".into(), "main".into())];
        assert_eq!(
            parse(tokens),
            Ok(vec![Node::Directive { pseudo_op: ".global".into(), symbol: "main".into() }]),
        );
    }

    #[test]
    fn parses_label() {
        let tokens = vec![Token::Label("main".into())];
        assert_eq!(parse(tokens), Ok(vec![Node::Label("main".into())]));
    }

    #[test]
    fn parses_instruction() {
        let tokens = vec![Token::Instruction("mov".into(), strings(&["rax", "42"]))];
        assert_eq!(
            parse(tokens),
            Ok(vec![Node::Instruction { opcode: "mov".into(), operands: strings(&["rax", "42"]) }]),
        );
    }

    #[test]
    fn preserves_statement_order() {
        let tokens = vec![
            Token::Directive(".global".into(), "main".into()),
            Token::Label("main".into()),
            Token::Instruction("ret".into(), vec![]),
        ];
        assert_eq!(
            parse(tokens),
            Ok(vec![
                Node::Directive { pseudo_op: ".global".into(), symbol: "main".into() },
                Node::Label("main".into()),
                Node::Instruction { opcode: "ret".into(), operands: vec![] },
            ]),
        );
    }

    #[test]
    fn rejects_global_without_symbol() {
        let tokens = vec![Token::Directive(".global".into(), "".into())];
        assert_eq!(
            parse(tokens),
            Err(ParseError::MissingSymbol { directive: ".global".into() }),
        );
    }
}
