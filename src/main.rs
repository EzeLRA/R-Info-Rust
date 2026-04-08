use std::fs;

/*
Para cada componente arreglar el error de compilacion para identificar
variables no declaradas y tokenizar gran parte de las lineas de codigo
*/

mod lib;
mod tests;

fn main() {
    let source = fs::read_to_string("src/tests/codigo.txt")
        .expect("Failed to read source file");
    let mut machine = lib::machine::Machine::new(source);
    machine.print_compilation();
}