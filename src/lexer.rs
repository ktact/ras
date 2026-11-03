use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Token {
    /// A name such as an opcode, register, symbol or directive (`.global`).
    Ident(String),
    Integer(String),
    Colon,
    Comma,
    Newline,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Ident(s) | Token::Integer(s) => write!(f, "'{}'", s),
            Token::Colon => write!(f, "':'"),
            Token::Comma => write!(f, "','"),
            Token::Newline => write!(f, "end of line"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct LexError {
    pub found: char,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unexpected character '{}'", self.found)
    }
}

impl std::error::Error for LexError {}

pub fn tokenize(input: &str) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();

    for line in input.lines() {
        tokenize_line(line, &mut tokens)?;
        tokens.push(Token::Newline);
    }

    Ok(tokens)
}

fn tokenize_line(line: &str, tokens: &mut Vec<Token>) -> Result<(), LexError> {
    let mut rest = line;

    loop {
        rest = rest.trim_start();
        let Some(c) = rest.chars().next() else {
            return Ok(());
        };

        let (token, len) = match c {
            ':' => (Token::Colon, 1),
            ',' => (Token::Comma, 1),
            c if c.is_ascii_digit() => {
                let len = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
                (Token::Integer(rest[..len].to_string()), len)
            }
            c if is_ident_char(c) => {
                let len = rest.find(|c: char| !is_ident_char(c)).unwrap_or(rest.len());
                (Token::Ident(rest[..len].to_string()), len)
            }
            found => return Err(LexError { found }),
        };

        tokens.push(token);
        rest = &rest[len..];
    }
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '.'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ident(s: &str) -> Token {
        Token::Ident(s.to_string())
    }

    #[test]
    fn tokenizes_directive_and_label() {
        assert_eq!(
            tokenize(".global main\nmain:"),
            Ok(vec![ident(".global"), ident("main"), Token::Newline, ident("main"), Token::Colon, Token::Newline]),
        );
    }

    #[test]
    fn tokenizes_instruction() {
        assert_eq!(
            tokenize("    mov rax, 42"),
            Ok(vec![ident("mov"), ident("rax"), Token::Comma, Token::Integer("42".into()), Token::Newline]),
        );
    }

    #[test]
    fn rejects_unexpected_character() {
        assert_eq!(tokenize("mov [rax]"), Err(LexError { found: '[' }));
    }
}
