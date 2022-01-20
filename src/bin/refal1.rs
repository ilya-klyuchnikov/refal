use refal::{compiler, data, vm1};
use std::{env, fs};

fn main() -> data::Result<()> {
    let args: Vec<String> = env::args().collect();
    let refal_file = &args[1];
    let goal = &args[2];
    let input = fs::read_to_string(refal_file).unwrap();
    let rasl_module = compiler::compile(&input)?;
    let defs = data::module_to_defs(rasl_module);
    let result = vm1::eval_main(&defs, goal);
    println!("{:?}", result);
    Ok(())
}
