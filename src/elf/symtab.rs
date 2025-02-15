use std::mem;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum SymType {
    NoType  = 0,
    Object  = 1,
    Func    = 2,
    Section = 3,
    File    = 4,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum SymBind {
    Local  = 0,
    Global = 1,
    Weak   = 2,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum SymVis {
    Default   = 0,
    Internal  = 1,
    Hidden    = 2,
    Protected = 3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64Sym {
    pub st_name: u32,
    pub st_info: u8,
    pub st_other: u8,
    pub st_shndx: u16,
    pub st_value: u64,
    pub st_size: u64,
}

impl Elf64Sym {
    pub fn new(st_name: u32, sym_type: SymType, sym_bind: SymBind, sym_vis: SymVis, st_shndx: u16, st_value: u64, st_size: u64) -> Self {
        Self {
            st_name,
            st_info: ((sym_bind as u8) << 4) | (sym_type as u8),
            st_other: sym_vis as u8,
            st_shndx,
            st_value,
            st_size,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                mem::size_of::<Elf64Sym>(),
            )
        }
    }
}

pub struct SymTab {
    pub symbols: Vec<Elf64Sym>,
}

impl SymTab {
    pub fn new() -> Self {
        let mut symbols = Vec::new();
        symbols.push(Elf64Sym::new(0, SymType::NoType, SymBind::Local, SymVis::Default, 0, 0, 0));

        Self { symbols }
    }

    pub fn add_symbol(&mut self, name_offset: u32, sym_type: SymType, sym_bind: SymBind, sym_vis: SymVis, shndx: u16, value: u64, size: u64) {
        self.symbols.push(Elf64Sym::new(name_offset, sym_type, sym_bind, sym_vis, shndx, value, size));
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        for sym in &self.symbols {
            data.extend_from_slice(sym.as_bytes());
        }
        data
    }
}
