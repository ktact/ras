use std::fmt;

use crate::parser::{Mnemonic, Node, Operand, Register, Statement};

#[derive(Debug, PartialEq)]
pub struct CodegenError {
    /// 1-based line number in the source.
    pub line: usize,
    pub kind: CodegenErrorKind,
}

#[derive(Debug, PartialEq)]
pub enum CodegenErrorKind {
    UnsupportedOperands { mnemonic: Mnemonic },
    ImmediateOutOfRange { value: i64 },
}

impl fmt::Display for CodegenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.kind)
    }
}

impl fmt::Display for CodegenErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodegenErrorKind::UnsupportedOperands { mnemonic } => {
                write!(f, "unsupported operands for '{}'", mnemonic)
            }
            CodegenErrorKind::ImmediateOutOfRange { value } => {
                write!(f, "immediate value {} is out of range", value)
            }
        }
    }
}

impl std::error::Error for CodegenError {}

fn encode_instruction(mnemonic: Mnemonic, operands: &[Operand]) -> Result<Vec<u8>, CodegenErrorKind> {
    match (mnemonic, operands) {
        (Mnemonic::Mov, [Operand::Register(Register::Rax), Operand::Immediate(imm)]) => {
            let imm8 = u8::try_from(*imm)
                .map_err(|_| CodegenErrorKind::ImmediateOutOfRange { value: *imm })?;
            Ok(vec![0x48, 0xc7, 0xc0, imm8, 0x00, 0x00, 0x00])
        }
        (Mnemonic::Ret, []) => Ok(vec![0xc3]),
        _ => Err(CodegenErrorKind::UnsupportedOperands { mnemonic }),
    }
}

pub fn generate_machine_code(ast: &[Statement]) -> Result<Vec<u8>, CodegenError> {
    let mut code = Vec::new();

    for statement in ast {
        if let Node::Instruction { mnemonic, operands } = &statement.node {
            let bytes = encode_instruction(*mnemonic, operands)
                .map_err(|kind| CodegenError { line: statement.line, kind })?;
            code.extend(bytes);
        }
    }

    Ok(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse;

    fn assemble(source: &str) -> Result<Vec<u8>, CodegenError> {
        let ast = parse(tokenize(source).expect("Failed to tokenize")).expect("Failed to parse");
        generate_machine_code(&ast)
    }

    #[test]
    fn encodes_mov_and_ret() {
        assert_eq!(
            assemble("mov rax, 42\nret"),
            Ok(vec![0x48, 0xc7, 0xc0, 42, 0x00, 0x00, 0x00, 0xc3]),
        );
    }

    #[test]
    fn rejects_unsupported_operands() {
        assert_eq!(
            assemble("ret\nret 1"),
            Err(CodegenError { line: 2, kind: CodegenErrorKind::UnsupportedOperands { mnemonic: Mnemonic::Ret } }),
        );
    }

    #[test]
    fn rejects_immediate_out_of_range() {
        assert_eq!(
            assemble("mov rax, 256"),
            Err(CodegenError { line: 1, kind: CodegenErrorKind::ImmediateOutOfRange { value: 256 } }),
        );
    }
}
