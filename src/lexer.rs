#[derive(Debug, PartialEq)]
pub enum Token {
    Directive(String, String),
    Label(String),
    Instruction(String, Vec<String>),
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }

        if line.starts_with('.') {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                let pseudo_op = parts[0].to_string();
                let symbol    = parts[1].to_string();
                tokens.push(Token::Directive(pseudo_op, symbol));
            }
        } else if line.ends_with(':') {
            tokens.push(Token::Label(line.trim_end_matches(':').to_string()));
        } else {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                let opcode   = parts[0].to_string();
                let operands = parts[1..].iter().map(|s| s.trim_end_matches(',').to_string()).collect();
                tokens.push(Token::Instruction(opcode, operands));
            }
        }
    }

    tokens
}
