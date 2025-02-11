use std::mem;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64Header {
    pub e_ident: [u8; 16],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

impl Elf64Header {
    pub fn new() -> Self {
        Self {
            e_ident: [
                /* MAGIC number */0x7f, b'E', b'L', b'F',
                /* EL_CLASS     */2,
                /* EI_DATA      */1,
                /* EI_VERSION   */1,
                /* EI_OSABI     */0,
                /* Padding      */0, 0, 0, 0, 0, 0, 0, 0,
            ],
            e_type: 1, // ET_REL
            e_machine: 0x3e, // EM_AMD64
            e_version: 1, // EV_CURRENT
            e_entry: 0x000000,
            e_phoff: 0,
            e_shoff: 0,
            e_flags: 0,
            e_ehsize:    mem::size_of::<Elf64Header>() as u16,
            e_phentsize: 0,
            e_phnum: 0,
            e_shentsize: 64,
            e_shnum: 0,
            e_shstrndx: 6,
        }
    }

    pub fn set_shoff(&mut self, offset: u64, section_count: u16) {
        self.e_shoff = offset;
        self.e_shnum = section_count;
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                mem::size_of::<Elf64Header>()
                )
        }
    }
}
