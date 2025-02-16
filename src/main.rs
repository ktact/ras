use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process;

mod elf;
use elf::header::Elf64Header;
use elf::section::{Elf64SectionHeader, SectionType, SectionFlags};
use elf::shstrtab::{ShStrTab};
use elf::symtab::{Elf64Sym, SymTab, SymType, SymBind, SymVis};
use elf::strtab::{StrTab};

#[derive(Debug, PartialEq)]
enum Token {
    Directive(String, String),
    Label(String),
    Instruction(String, Vec<String>),
}

fn tokenize(input: &str) -> Vec<Token> {
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

#[derive(Debug)]
enum Node {
    Directive { pseudo_op: String, symbol: String },
    Label(String),
    Instruction { opcode: String, operands: Vec<String> },
}

fn parse(tokens: Vec<Token>) -> Vec<Node> {
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

fn generate_machine_code(ast: &[Node]) -> Vec<u8> {
    let mut code = Vec::new();

    for node in ast {
        if let Node::Instruction { opcode, operands } = node {
            code.extend(encode_instruction(&opcode, &operands));
        }
    }

    code
}

fn main() -> std::io::Result<()> {
    let mut args = env::args().skip(1);
    let mut input_file: Option<PathBuf> = None;
    let mut output_file = String::from("a.out"); // Default output file

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-c" => {
                input_file = args.next().map(PathBuf::from);
            }
            "-o" => {
                output_file = args.next().unwrap_or_else(|| {
                    eprintln!("Error: -o requires a filename");
                    std::process::exit(1);
                });
            }
            _ => {
                if input_file.is_none() {
                    input_file = Some(PathBuf::from(arg));
                } else {
                    eprintln!("Error: Unexpected argument '{}'", arg);
                    std::process::exit(1);
                }
            }
        }
    }

    let input_file = input_file.expect("No input file specified");

    let source = fs::read_to_string(&input_file).unwrap_or_else(|err| {
        eprintln!("Error: Failed to read file {}: {}", input_file.display(), err);
        process::exit(1);
    });

    let tokens = tokenize(&source);
    let ast = parse(tokens);
    let machine_code = generate_machine_code(&ast);

    #[cfg(FALSE)]
    {
        println!("Generated Machine Code:");
        for (i, byte) in machine_code.iter().enumerate() {
            print!("{:02X} ", byte);
            if (i + 1) % 16 == 0 {
                println!();
            }
        }
        println!();
    }

    let section_names = vec![
        ".symtab",
        ".strtab",
        ".shstrtab",
        ".text",
        ".data",
        ".bss",
    ];
    let shstrtab = ShStrTab::new(&section_names);
    #[cfg(FALSE)]
    {
        println!("{:?}", shstrtab.as_bytes().iter().map(|b| format!("0x{:02X}", b)).collect::<Vec<_>>());
    }

    let mut strtab = StrTab::new();
    let mut symtab = SymTab::new();
    for node in ast {
        if let Node::Directive { pseudo_op, symbol } = node {
            if pseudo_op == ".global" {
                strtab.add_str(&symbol);
                symtab.add_symbol(
                    strtab.get_offset_by(&symbol).unwrap_or(0),
                    SymType::NoType,
                    SymBind::Global,
                    SymVis::Default,
                    1,
                    0x0000000000000000,
                    0
                );
            }
        }
    }

    let mut elf_header = Elf64Header::new();
    let mut shs: Vec<Elf64SectionHeader> = Vec::new();

    // NULL Section Header
    let null_sh = Elf64SectionHeader::new(SectionType::Null, 0 as u64, 0);
    shs.push(null_sh);

    // .text Section Header
    let mut text_sh = Elf64SectionHeader::new(SectionType::ProgBits, SectionFlags::Alloc as u64 | SectionFlags::ExecInstr as u64, 0x40);
    if let Some(offset) = shstrtab.get_offset_by(".text") {
        text_sh.set_name_index(offset);
    }
    text_sh.set_size(machine_code.len() as u64);
    shs.push(text_sh);

    // .data Section Header
    let mut data_sh = Elf64SectionHeader::new(SectionType::ProgBits, SectionFlags::Write as u64 | SectionFlags::Alloc as u64, 0x40 + machine_code.len() as u64);
    if let Some(offset) = shstrtab.get_offset_by(".data") {
        data_sh.set_name_index(offset);
    }
    shs.push(data_sh);

    // .bss Section Header
    let mut bss_sh = Elf64SectionHeader::new(SectionType::NoBits,   SectionFlags::Write as u64 | SectionFlags::Alloc as u64, 0x40 + machine_code.len() as u64);
    if let Some(offset) = shstrtab.get_offset_by(".bss") {
        bss_sh.set_name_index(offset);
    }
    shs.push(bss_sh);

    // .symtab Section Header
    let symtab_offset = (elf_header.as_bytes().len() + machine_code.len()).try_into().unwrap();
    let mut symtab_sh = Elf64SectionHeader::new(SectionType::SymTab,   0 as u64, symtab_offset);
    if let Some(offset) = shstrtab.get_offset_by(".symtab") {
        symtab_sh.set_name_index(offset);
    }
    symtab_sh.set_size(symtab.as_bytes().len() as u64);
    shs.push(symtab_sh);

    // .strtab Section Header
    let strtab_offset = symtab_offset + symtab.as_bytes().len() as u64;
    let mut strtab_sh = Elf64SectionHeader::new(SectionType::StrTab,   0 as u64, strtab_offset);
    if let Some(offset) = shstrtab.get_offset_by(".strtab") {
        strtab_sh.set_name_index(offset);
    }
    strtab_sh.set_size(strtab.as_bytes().len() as u64);
    shs.push(strtab_sh);

    // .shstrtab Section Header
    let shstrtab_offset = strtab_offset + strtab.as_bytes().len() as u64;
    let mut shstrtab_sh = Elf64SectionHeader::new(SectionType::StrTab,   0 as u64, shstrtab_offset);
    if let Some(offset) = shstrtab.get_offset_by(".shstrtab") {
        shstrtab_sh.set_name_index(offset);
    }
    shstrtab_sh.set_size(shstrtab.as_bytes().len() as u64);
    shs.push(shstrtab_sh);

    elf_header.set_shoff(0xb0, shs.len() as u16);

    let mut output = File::create(output_file)?;

    output.write_all(elf_header.as_bytes())?;
    output.write_all(&machine_code)?;
    output.write_all(&symtab.as_bytes())?;
    output.write_all(&strtab.as_bytes())?;
    output.write_all(&shstrtab.as_bytes())?;
    let padding_size = 6;
    output.write_all(&vec![0; padding_size as usize]).expect("Failed to write padding");

    for sh in &shs {
        output.write_all(sh.as_bytes()).expect("Failed to write section header");
    }

    Ok(())
}
