use crate::parser::{Mnemonic, Node, Operand, Register};

fn encode_instruction(mnemonic: Mnemonic, operands: &[Operand]) -> Vec<u8> {
    match (mnemonic, operands) {
        (Mnemonic::Mov, [Operand::Register(Register::Rax), Operand::Immediate(imm)]) => {
            let imm8 = u8::try_from(*imm).expect("Invalid immediate value");
            vec![0x48, 0xc7, 0xc0, imm8, 0x00, 0x00, 0x00]
        }
        (Mnemonic::Ret, []) => vec![0xc3],
        _ => panic!("Unsupported operands for {:?}", mnemonic),
    }
}

pub fn generate_machine_code(ast: &[Node]) -> Vec<u8> {
    let mut code = Vec::new();

    for node in ast {
        if let Node::Instruction { mnemonic, operands } = node {
            code.extend(encode_instruction(*mnemonic, operands));
        }
    }

    code
}
