use refal::{compiler, data, parser};
use std::io::Write;
use std::path::Path;
use std::{env, fs};

fn main() -> data::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: refalc <refal_file>");
        std::process::exit(1);
    }

    let refal_file = &args[1];
    let input = fs::read_to_string(refal_file).unwrap();

    // Parse the module to get the original function order
    let module = parser::parse_input(&input)?;
    let module_name = &module.name;

    // Get function names in the original order
    let function_names: Vec<String> = module
        .functions
        .iter()
        .map(|f| compiler::qualify(module_name, &f.name))
        .collect();

    // Compile to get the commands
    let defs = compiler::compile_module(&module);

    // Determine output filename
    let input_path = Path::new(refal_file);
    let stem = input_path.file_stem().unwrap().to_str().unwrap();
    let output_path = input_path.with_file_name(format!("{}.rasl", stem));

    // Format and write the bytecode
    let mut output = fs::File::create(output_path).unwrap();

    writeln!(
        output,
        "# Refal Assembly Language (RASL) generated from {}",
        refal_file
    )
    .unwrap();
    writeln!(output, "# Module: {}", module_name).unwrap();
    writeln!(output).unwrap();

    // Format and write each function's bytecode in the original order
    for name in &function_names {
        if let Some(commands) = defs.get(name) {
            writeln!(output, "Function: {}", name).unwrap();

            for (i, cmd) in commands.iter().enumerate() {
                writeln!(output, "  {:4}: {:?}", i, cmd).unwrap();
            }

            writeln!(output).unwrap();
        }
    }

    Ok(())
}
