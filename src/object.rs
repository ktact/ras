use crate::elf::header::Elf64Header;
use crate::elf::section::{Elf64SectionHeader, SectionType, SectionFlags};
use crate::elf::shstrtab::{ShStrTab};
use crate::elf::symtab::{SymTab, SymType, SymBind, SymVis};
use crate::elf::strtab::{StrTab};
use crate::codegen::MachineCode;
use crate::parser::{Node, Statement};

struct Symbol {
    name: String,
    global: bool,
}

/// Lists the symbols in the order GNU as writes them to .symtab:
///   in order of first appearance with all locals before globals.
fn collect_symbols(ast: &[Statement]) -> Vec<Symbol> {
    let mut symbols: Vec<Symbol> = Vec::new();

    for statement in ast {
        let (name, global) = match &statement.node {
            Node::Directive { pseudo_op, symbol } if pseudo_op == ".global" => (symbol, true),
            // Labels starting with .L are local to the assembler and never become symbols.
            Node::Label(name) if !name.starts_with(".L") => (name, false),
            _ => continue,
        };

        match symbols.iter_mut().find(|symbol| symbol.name == *name) {
            Some(symbol) => symbol.global |= global,
            None => symbols.push(Symbol { name: name.clone(), global }),
        }
    }

    symbols.sort_by_key(|symbol| symbol.global);
    symbols
}

pub fn build_object(ast: &[Statement], machine_code: &MachineCode) -> Vec<u8> {
    let mut section_names = vec![
        ".shstrtab",
        ".text",
        ".data",
        ".bss",
    ];
    let mut strtab = StrTab::new();
    let mut symtab = SymTab::new();

    let symbols = collect_symbols(ast);
    if !symbols.is_empty() {
        section_names.insert(0, ".strtab");
        section_names.insert(0, ".symtab");

        for symbol in &symbols {
            let name_offset = strtab.add_str(&symbol.name);
            let bind = if symbol.global { SymBind::Global } else { SymBind::Local };

            // A symbol that is never defined as a label is undefined (section index 0).
            let (shndx, value) = match machine_code.labels.get(&symbol.name) {
                Some(&offset) => (1, offset),
                None => (0, 0),
            };
            symtab.add_symbol(name_offset, SymType::NoType, bind, SymVis::Default, shndx, value, 0);
        }
    }
    let shstrtab = ShStrTab::new(&section_names);

    let alloc = SectionFlags::Alloc as u64;
    let write = SectionFlags::Write as u64;
    let exec  = SectionFlags::ExecInstr as u64;

    // (name, type, flags, contents) in section header order
    let mut sections: Vec<(&str, SectionType, u64, Vec<u8>)> = vec![
        (".text", SectionType::ProgBits, alloc | exec,  machine_code.text.clone()),
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
        let mut sh = Elf64SectionHeader::new(*sh_type, *flags, 0);
        sh_offset = align_up(sh_offset, sh.sh_addralign);
        sh.set_offset(sh_offset);
        if let Some(offset) = shstrtab.get_offset_by(name) {
            sh.set_name_index(offset);
        }
        sh.set_size(contents.len() as u64);
        if let SectionType::SymTab = sh_type {
            // .symtab's sh_link points to the index of .strtab (+1 for the NULL section).
            let strtab_index = sections.iter().position(|(n, ..)| *n == ".strtab").unwrap() + 1;
            sh.set_link(strtab_index as u32);

            // .symtab's sh_info is the index of the first global symbol (+1 for the NULL symbol).
            let local_count = symbols.iter().filter(|symbol| !symbol.global).count();
            sh.set_info((local_count + 1) as u32);
        }
        shs.push(sh);

        sh_offset = sh_offset + contents.len() as u64;
    }

    // Section headers are aligned to 8 bytes, as GNU as does for ELF64.
    let shoff = align_up(sh_offset, 8);

    elf_header.set_shoff(shoff, shs.len() as u16);
    // .shstrtab is always the last section.
    elf_header.set_shstrndx((shs.len() - 1) as u16);

    let mut output = elf_header.as_bytes().to_vec();
    for ((_, _, _, contents), sh) in sections.iter().zip(&shs[1..]) {
        output.resize(sh.sh_offset as usize, 0);
        output.extend_from_slice(contents);
    }
    output.resize(shoff as usize, 0);

    for sh in &shs {
        output.extend_from_slice(sh.as_bytes());
    }

    output
}

fn align_up(offset: u64, align: u64) -> u64 {
    if align <= 1 {
        return offset;
    }
    offset.div_ceil(align) * align
}
