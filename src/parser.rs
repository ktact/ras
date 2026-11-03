use crate::lexer::Token;

#[derive(Debug, PartialEq)]
pub enum Node {
    Directive { pseudo_op: String, symbol: String },
    Label(String),
    Instruction { opcode: String, operands: Vec<String> },
}

pub fn parse(tokens: Vec<Token>) -> Vec<Node> {
    let mut tree = Vec::new();

    for token in tokens {
        match token {
            Token::Directive(pseudo_op, symbol) => {
                tree.push(Node::Directive { pseudo_op, symbol });
            }
            Token::Label(label) => tree.push(Node::Label(label)),
            Token::Instruction(opcode, operands) =>  {
                tree.push(Node::Instruction { opcode, operands });
            }
        }
    }

    tree
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parses_empty_input() {
        assert_eq!(parse(vec![]), vec![]);
    }

    #[test]
    fn parses_directive() {
        let tokens = vec![Token::Directive(".global".into(), "main".into())];
        assert_eq!(
            parse(tokens),
            vec![Node::Directive { pseudo_op: ".global".into(), symbol: "main".into() }],
        );
    }

    #[test]
    fn parses_label() {
        let tokens = vec![Token::Label("main".into())];
        assert_eq!(parse(tokens), vec![Node::Label("main".into())]);
    }

    #[test]
    fn parses_instruction() {
        let tokens = vec![Token::Instruction("mov".into(), strings(&["rax", "42"]))];
        assert_eq!(
            parse(tokens),
            vec![Node::Instruction { opcode: "mov".into(), operands: strings(&["rax", "42"]) }],
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
            vec![
                Node::Directive { pseudo_op: ".global".into(), symbol: "main".into() },
                Node::Label("main".into()),
                Node::Instruction { opcode: "ret".into(), operands: vec![] },
            ],
        );
    }
}
