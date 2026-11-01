use crate::parser::Node;

fn encode_instruction(opcode: &str, operands: &[String]) -> Vec<u8> {
    match (opcode, operands) {
        ("mov", [reg, imm]) if reg == "rax" => {
            let imm64: u8 = imm.parse().expect("Invalid immediate value");
            vec![0x48, 0xc7, 0xc0, imm64, 0x00, 0x00, 0x00]
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
