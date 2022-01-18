use refal::{compiler, data, vm};
use std::{env, fs};

fn main() -> data::Result<()> {
    let args: Vec<String> = env::args().collect();
    let refal_file = &args[1];
    let goal = &args[2];
    let input = fs::read_to_string(refal_file).unwrap();
    let rasl_module = compiler::compile(&input)?;
    let defs = data::module_to_defs(rasl_module);
    let result = vm::eval_main(&defs, goal);
    println!("{:?}", result);
    Ok(())
}
