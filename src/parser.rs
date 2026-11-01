use crate::lexer::Token;

#[derive(Debug)]
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
