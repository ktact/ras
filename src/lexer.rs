use std::fmt;

#[derive(Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    /// 1-based line number in the source.
    pub line: usize,
}

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    /// A name such as an opcode, register, symbol or directive (`.global`).
    Ident(String),
    Integer(String),
    Colon,
    Comma,
    Newline,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Ident(s) | TokenKind::Integer(s) => write!(f, "'{}'", s),
            TokenKind::Colon => write!(f, "':'"),
            TokenKind::Comma => write!(f, "','"),
            TokenKind::Newline => write!(f, "end of line"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct LexError {
    pub line: usize,
    pub found: char,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: unexpected character '{}'", self.line, self.found)
    }
}

impl std::error::Error for LexError {}

pub fn tokenize(input: &str) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();

    for (index, text) in input.lines().enumerate() {
        let line = index + 1;
        tokenize_line(text, line, &mut tokens)?;
        tokens.push(Token { kind: TokenKind::Newline, line });
    }

    Ok(tokens)
}

fn tokenize_line(text: &str, line: usize, tokens: &mut Vec<Token>) -> Result<(), LexError> {
    let mut rest = text;

    loop {
        rest = rest.trim_start();
        let Some(c) = rest.chars().next() else {
            return Ok(());
        };

        let (kind, len) = match c {
            ':' => (TokenKind::Colon, 1),
            ',' => (TokenKind::Comma, 1),
            c if c.is_ascii_digit() => {
                let len = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
                (TokenKind::Integer(rest[..len].to_string()), len)
            }
            c if is_ident_char(c) => {
                let len = rest.find(|c: char| !is_ident_char(c)).unwrap_or(rest.len());
                (TokenKind::Ident(rest[..len].to_string()), len)
            }
            found => return Err(LexError { line, found }),
        };

        tokens.push(Token { kind, line });
        rest = &rest[len..];
    }
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '.'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(source: &str) -> Vec<TokenKind> {
        tokenize(source).expect("Failed to tokenize").into_iter().map(|token| token.kind).collect()
    }

    fn ident(s: &str) -> TokenKind {
        TokenKind::Ident(s.to_string())
    }

    #[test]
    fn tokenizes_directive_and_label() {
        assert_eq!(
            kinds(".global main\nmain:"),
            vec![ident(".global"), ident("main"), TokenKind::Newline, ident("main"), TokenKind::Colon, TokenKind::Newline],
        );
    }

    #[test]
    fn tokenizes_instruction() {
        assert_eq!(
            kinds("    mov rax, 42"),
            vec![ident("mov"), ident("rax"), TokenKind::Comma, TokenKind::Integer("42".into()), TokenKind::Newline],
        );
    }

    #[test]
    fn records_line_numbers() {
        let lines: Vec<usize> = tokenize("main:\n\n    ret")
            .expect("Failed to tokenize")
            .iter()
            .map(|token| token.line)
            .collect();
        // main : \n | \n | ret \n
        assert_eq!(lines, vec![1, 1, 1, 2, 3, 3]);
    }

    #[test]
    fn rejects_unexpected_character() {
        assert_eq!(tokenize("ret\nmov [rax]"), Err(LexError { line: 2, found: '[' }));
    }
}
