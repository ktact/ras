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
use elf::symtab::{SymTab, SymType, SymBind, SymVis};
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

    let mut section_names = vec![
        ".shstrtab",
        ".text",
        ".data",
        ".bss",
    ];
    let mut strtab = StrTab::new();
    let mut symtab = SymTab::new();

    if !machine_code.is_empty() {
        section_names.insert(0, ".strtab");
        section_names.insert(0, ".symtab");

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
    }
    let shstrtab = ShStrTab::new(&section_names);

    let alloc = SectionFlags::Alloc as u64;
    let write = SectionFlags::Write as u64;
    let exec  = SectionFlags::ExecInstr as u64;

    // (name, type, flags, contents) in section header order
    let mut sections: Vec<(&str, SectionType, u64, Vec<u8>)> = vec![
        (".text", SectionType::ProgBits, alloc | exec,  machine_code.clone()),
        (".data", SectionType::ProgBits, write | alloc, Vec::new()),
        (".bss",  SectionType::NoBits,   write | alloc, Vec::new()),
    ];
    if section_names.contains(&".symtab") {
        sections.push((".symtab", SectionType::SymTab, 0, symtab.as_bytes()));
        sections.push((".strtab", SectionType::StrTab, 0, strtab.as_bytes().to_vec()));
    }
    sections.push((".shstrtab", SectionType::StrTab, 0, shstrtab.as_bytes().to_vec()));

    let mut elf_header = Elf64Header::new();
    let mut shs = vec![Elf64SectionHeader::new(SectionType::Null, 0, 0)];
    let mut sh_offset = elf_header.as_bytes().len() as u64;

    for (name, sh_type, flags, contents) in &sections {
        let mut sh = Elf64SectionHeader::new(*sh_type, *flags, sh_offset);
        if let Some(offset) = shstrtab.get_offset_by(name) {
            sh.set_name_index(offset);
        }
        sh.set_size(contents.len() as u64);
        if let SectionType::SymTab = sh_type {
            // .symtab's sh_link points to the index of .strtab (+1 for the NULL section).
            let strtab_index = sections.iter().position(|(n, ..)| *n == ".strtab").unwrap() + 1;
            sh.set_link(strtab_index as u32);
        }
        shs.push(sh);

        sh_offset = sh_offset + contents.len() as u64;
    }

    let padding_size = 16 - (sh_offset % 16);
    let shoff = sh_offset + padding_size;

    elf_header.set_shoff(shoff, shs.len() as u16);
    // .shstrtab is always the last section.
    elf_header.set_shstrndx((shs.len() - 1) as u16);

    let mut output = File::create(output_file)?;

    output.write_all(elf_header.as_bytes())?;
    for (_, _, _, contents) in &sections {
        output.write_all(contents)?;
    }
    output.write_all(&vec![0; padding_size as usize]).expect("Failed to write padding");

    for sh in &shs {
        output.write_all(sh.as_bytes()).expect("Failed to write section header");
    }

    Ok(())
}
