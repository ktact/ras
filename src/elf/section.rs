use std::mem;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum SectionType {
    Null = 0,
    ProgBits = 1,
    SymTab = 2,
    StrTab = 3,
    Rela = 4,
    Hash = 5,
    Dynamic = 6,
    Note = 7,
    NoBits = 8,
    Rel = 9,
    ShLib = 10,
    DynSym = 11,
}

#[repr(u64)]
#[derive(Debug, Clone, Copy)]
pub enum SectionFlags {
    Write = 0x1,
    Alloc = 0x2,
    ExecInstr = 0x4,
    Merge = 0x10,
    Strings = 0x20,
    InfoLink = 0x40,
    LinkOrder = 0x80,
    OsNonConforming = 0x100,
    Group = 0x200,
    Tls = 0x400,
}
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64SectionHeader {
    pub sh_name: u32,
    pub sh_type: SectionType,
    pub sh_flags: u64,
    pub sh_addr: u64,
    pub sh_offset: u64,
    pub sh_size: u64,
    pub sh_link: u32,
    pub sh_info: u32,
    pub sh_addralign: u64,
    pub sh_entsize: u64,
}

impl Elf64SectionHeader {
    pub fn new(sh_type: SectionType, sh_flags: u64, sh_offset: u64) -> Self {
        let sh_addralign = match sh_type {
            SectionType::Null => 0,
            SectionType::SymTab => 8,
            _ => 1,
        };

        let sh_link = match sh_type {
            // Symtab.sh_link should point to the index of StrTab.
            SectionType::SymTab => 5,
            _ => 0,
        };

        let sh_info = match sh_type {
            SectionType::SymTab => 1,
            _ => 0,
        };

        let sh_entsize = match sh_type {
            SectionType::SymTab => 24,
            _ => 0,
        };

        Self {
            sh_name: 0,
            sh_type,
            sh_flags,
            sh_addr: 0,
            sh_offset,
            sh_size: 0,
            sh_link,
            sh_info,
            sh_addralign,
            sh_entsize,
        }
    }

    pub fn set_name_index(&mut self, name_index: u32) {
        self.sh_name = name_index;
    }

    pub fn set_size(&mut self, size: u64) {
        self.sh_size = size;
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                mem::size_of::<Elf64SectionHeader>(),
                )
        }
    }
}

