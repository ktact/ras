use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

use ras::codegen::generate_machine_code;
use ras::lexer::tokenize;
use ras::object::build_object;
use ras::parser::parse;

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

    let input_file = input_file.unwrap_or_else(|| {
        eprintln!("Error: No input file specified");
        process::exit(1);
    });

    let source = fs::read_to_string(&input_file).unwrap_or_else(|err| {
        eprintln!("Error: Failed to read file {}: {}", input_file.display(), err);
        process::exit(1);
    });

    let tokens = tokenize(&source);
    let ast = parse(tokens);
    let machine_code = generate_machine_code(&ast);
    let object = build_object(&ast, &machine_code);

    fs::write(output_file, object)
}
