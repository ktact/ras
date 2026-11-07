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

fn compare_object_files(input_name: &str) {
    let asm_file = &Path::new("tests/inputs").join(input_name);
    let dir = tempdir().expect("Failed to create temp directory");

    // gcc's output is the reference
    let gcc_object_bytes = assemble("gcc", asm_file, &dir.path().join("gcc.o"));
    let ras_object_bytes = assemble(env!("CARGO_BIN_EXE_ras"), asm_file, &dir.path().join("ras.o"));

    assert_eq!(gcc_object_bytes, ras_object_bytes, "Object files do not match!");
}

#[test]
fn test_42() {
    compare_object_files("42.s");
}

#[test]
fn test_empty_file() {
    compare_object_files("empty.s");
}

#[test]
fn test_shoff_already_aligned() {
    compare_object_files("shoff_already_aligned.s");
}

#[test]
fn test_shoff_aligned_to_8() {
    compare_object_files("shoff_aligned_to_8.s");
}

#[test]
fn test_symtab_alignment() {
    compare_object_files("symtab_alignment.s");
}

#[test]
fn test_labels() {
    compare_object_files("labels.s");
}

#[test]
fn test_no_symbols() {
    compare_object_files("no_symbols.s");
}

#[test]
fn test_undefined_global() {
    compare_object_files("undefined_global.s");
}

#[test]
fn test_mov_imm_registeres() {
    compare_object_files("mov_imm_registers.s");
}

#[test]
fn test_mov_imm32() {
    compare_object_files("mov_imm32.s");
}

#[test]
fn test_push_register() {
    compare_object_files("push_register.s");
}

#[test]
fn test_pop_register() {
    compare_object_files("pop_register.s");
}

#[test]
fn test_push_imm32() {
    compare_object_files("push_imm32.s");
}

#[test]
fn test_push_imm8() {
    compare_object_files("push_imm8.s");
}

#[test]
fn test_add_imm8() {
    compare_object_files("add_imm8.s");
}
