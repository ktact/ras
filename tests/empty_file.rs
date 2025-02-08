use std::fs;
use std::process::Command;
use std::io::Read;
use tempfile::tempdir;
use std::env;

#[test]
fn test_object_generation_for_empty_file() {
    let dir = tempdir().expect("Failed to create temp directory");
    let empty_asm = dir.path().join("empty.s");
    let gcc_object_file = dir.path().join("empty.gcc.o");

    // Create an empty assembly file
    fs::File::create(&empty_asm).expect("Failed to create empty.s");

    // Generate refeence object file by GCC
    let gcc_status = Command::new("gcc")
        .arg("-c")
        .arg(&empty_asm)
        .arg("-o")
        .arg(&gcc_object_file)
        .status()
        .expect("Failed to run gcc");

    assert!(gcc_status.success(), "gcc failed to generate object file");

    // Generate object file by ras
    let ras_status = Command::new("target/debug/ras")
        .status()
        .expect("Failed to run ras");

    assert!(ras_status.success(), "ras failed to generate object file");

    let ras_object_file = env::current_dir().unwrap().join("empty.o");
    assert!(ras_object_file.exists(), "Expected assembler output file '{}' was not found", ras_object_file.display());

    let mut gcc_object_bytes = Vec::new();
    let mut ras_object_bytes = Vec::new();

    fs::File::open(&gcc_object_file)
        .expect("Failed to open gcc output file")
        .read_to_end(&mut gcc_object_bytes)
        .expect("Failed to read gcc output file");
    fs::File::open(&ras_object_file)
        .expect("Failed to open ras output file")
        .read_to_end(&mut ras_object_bytes)
        .expect("Failed to read ras output file");

    assert_eq!(gcc_object_bytes, ras_object_bytes, "Object files do not match!");

    fs::remove_file(&ras_object_file).expect("Failed to delete assembler output file");
}
