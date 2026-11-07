//! Encodes instructions into x86-64 machine code.
//!
//! Section, table and figure numbers in comments refer to the Intel® 64 and IA-32
//! Architectures Software Developer's Manual, Volume 2: Instruction Set Reference
//! (order number 325383-093US, September 2026), available from
//! <https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html>

use std::collections::HashMap;
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

/// Returns a REX prefix with W=1 (64-bit operand size) for an instruction whose
/// ModRM r/m field names `rm`.
///
/// The REX prefix is laid out as `0100 WRXB`:
///
/// - W: 1 selects a 64-bit operand size, hence the base value 0x48.
/// - B: the 4th bit of the register number in ModRM.rm, which only has 3 bits.
///
/// | register | number | binary | REX.B | ModRM.rm |
/// |----------|--------|--------|-------|----------|
/// | rdi      | 7      | 0111   | 0     | 111      |
/// | r8       | 8      | 1000   | 1     | 000      |
/// | r15      | 15     | 1111   | 1     | 111      |
///
/// So `rdi` and `r15` share the same ModRM.rm and are told apart only by REX.B
/// (0x48 vs 0x49).
///
/// See 2.2.1 "REX Prefixes", Table 2-4 and Figure 2-5.
fn rex_w(rm: Register) -> u8 {
    0x48 | (rm as u8 >> 3)
}

/// Returns a ModRM byte whose r/m field names the register `rm`.
///
/// The ModRM byte is laid out as `mod(2) reg(3) rm(3)`:
///
/// - mod: 11 (0xc0) makes r/m name a register rather than a memory operand.
/// - reg: either another register number or an opcode extension.
///        For example, `C7 /0` in the manual means reg must be 0.
/// - rm: the low 3 bits of the register number. The 4th bit goes to REX.B (see [`rex_w`]).
///
/// See 2.1.3 "ModR/M and SIB Bytes" and Table 2-2. The "/digit" notation is
/// described in 3.1.1.1.
fn modrm_register(reg: u8, rm: Register) -> u8 {
    0xc0 | (reg << 3) | (rm as u8 & 0b111)
}

fn encode_instruction(mnemonic: Mnemonic, operands: &[Operand]) -> Result<Vec<u8>, CodegenErrorKind> {
    match (mnemonic, operands) {
        (Mnemonic::Mov, [Operand::Register(dst), Operand::Immediate(imm)]) => {
            let imm8 = u8::try_from(*imm)
                .map_err(|_| CodegenErrorKind::ImmediateOutOfRange { value: *imm })?;
            // REX.W + C7 /0 id: MOV r/m64, imm32 (see "MOV—Move")
            Ok(vec![rex_w(*dst), 0xc7, modrm_register(0, *dst), imm8, 0x00, 0x00, 0x00])
        }
        (Mnemonic::Ret, []) => Ok(vec![0xc3]),
        _ => Err(CodegenErrorKind::UnsupportedOperands { mnemonic }),
    }
}

#[derive(Debug, PartialEq)]
pub struct MachineCode {
    pub text: Vec<u8>,
    /// Offset of each label from the start of .text.
    pub labels: HashMap<String, u64>,
}

pub fn generate_machine_code(ast: &[Statement]) -> Result<MachineCode, CodegenError> {
    let mut text = Vec::new();
    let mut labels = HashMap::new();

    for statement in ast {
        match &statement.node {
            Node::Label(name) => {
                labels.insert(name.clone(), text.len() as u64);
            }
            Node::Instruction { mnemonic, operands } => {
                let bytes = encode_instruction(*mnemonic, operands)
                    .map_err(|kind| CodegenError { line: statement.line, kind })?;
                text.extend(bytes);
            }
            Node::Directive { .. } => {}
        }
    }

    Ok(MachineCode { text, labels })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse;

    fn assemble(source: &str) -> Result<MachineCode, CodegenError> {
        let ast = parse(tokenize(source).expect("Failed to tokenize")).expect("Failed to parse");
        generate_machine_code(&ast)
    }

    #[test]
    fn encodes_mov_and_ret() {
        assert_eq!(
            assemble("mov rax, 42\nret").map(|machine_code| machine_code.text),
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

    #[test]
    fn records_label_offsets() {
        let labels = assemble("a:\n    ret\nb:\n    mov rax, 1\nc:").expect("Failed to assemble").labels;
        assert_eq!(labels["a"], 0);
        assert_eq!(labels["b"], 1);
        assert_eq!(labels["c"], 8);
    }
}
