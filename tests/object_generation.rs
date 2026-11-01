use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

/// Assembles `asm_file` with `assembler` and returns the bytes of the resulting object file.
fn assemble(assembler: &str, asm_file: &Path, object_file: &Path) -> Vec<u8> {
    let status = Command::new(assembler)
        .arg("-c")
        .arg(asm_file)
        .arg("-o")
        .arg(object_file)
        .status()
        .unwrap_or_else(|err| panic!("Failed to run {}: {}", assembler, err));

    assert!(status.success(), "{} failed to generate object file", assembler);

    fs::read(object_file).unwrap_or_else(|err| panic!("Failed to read {}: {}", object_file.display(), err))
}

fn compare_object_files(asm_file: &Path) {
    let dir = tempdir().expect("Failed to create temp directory");

    // gcc's output is the reference
    let gcc_object_bytes = assemble("gcc", asm_file, &dir.path().join("gcc.o"));
    let ras_object_bytes = assemble(env!("CARGO_BIN_EXE_ras"), asm_file, &dir.path().join("ras.o"));

    assert_eq!(gcc_object_bytes, ras_object_bytes, "Object files do not match!");
}

#[test]
fn test_42() {
    compare_object_files(Path::new("tests/inputs/42.s"));
}

#[test]
fn test_empty_file() {
    compare_object_files(Path::new("tests/inputs/empty.s"));
}

#[test]
fn test_shoff_already_aligned() {
    compare_object_files(Path::new("tests/inputs/shoff_already_aligned.s"));
}

#[test]
fn test_shoff_aligned_to_8() {
    compare_object_files(Path::new("tests/inputs/shoff_aligned_to_8.s"));
}

#[test]
fn test_symtab_alignment() {
    compare_object_files(Path::new("tests/inputs/symtab_alignment.s"));
}
