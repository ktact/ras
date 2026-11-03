use crate::parser::{Node, Operand, Register};

fn encode_instruction(opcode: &str, operands: &[Operand]) -> Vec<u8> {
    match (opcode, operands) {
        ("mov", [Operand::Register(Register::Rax), Operand::Immediate(imm)]) => {
            let imm8 = u8::try_from(*imm).expect("Invalid immediate value");
            vec![0x48, 0xc7, 0xc0, imm8, 0x00, 0x00, 0x00]
        }
        ("ret", []) => vec![0xc3],
        _ => panic!("Unknown instruction"),
    }
}

pub fn generate_machine_code(ast: &[Node]) -> Vec<u8> {
    let mut code = Vec::new();

    for node in ast {
        if let Node::Instruction { opcode, operands } = node {
            code.extend(encode_instruction(&opcode, &operands));
        }
    }

    code
}
