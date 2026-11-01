pub mod header;
pub mod section;
pub mod shstrtab;
pub mod symtab;
pub mod strtab;

/// Views a `#[repr(C)]` ELF structure as its raw bytes.
pub(crate) fn struct_as_bytes<T: Copy>(value: &T) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(
            (value as *const T) as *const u8,
            std::mem::size_of::<T>(),
            )
    }
}
