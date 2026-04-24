use std::fs;
use std::path::Path;

mod ast;
mod macros;
mod parser;
mod traits;
mod translator;

use parser::Parser;
use translator::Translator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <input.bt> [output.hexpat]", args[0]);
        std::process::exit(1);
    }

    let input_file = &args[1];
    let path = Path::new(input_file);
    let filename = format!("{}.hexpat", path.file_stem().unwrap().to_string_lossy());
    let output_file = args.get(2).map(|s| s.as_str()).unwrap_or_else(|| &filename);

    // Read the 010 template
    let content = fs::read_to_string(input_file)?;

    // Parse the template
    let mut parser = Parser::new(&content);
    let mut template = parser.parse()?;

    // Convert to ImHex format
    let mut converter = Translator::new();
    let hexpat = converter.translate(&mut template)?;

    // Write output
    fs::write(output_file, hexpat)?;
    println!("Successfully converted {} to {}", input_file, output_file);

    Ok(())
}
