use sim86::disassemble;
use std::{env, fs};

fn main() {
    let args: Vec<String> = env::args().collect();
    let (name, files) = args.split_first().unwrap();

    if files.is_empty() {
        eprintln!("\nAt least one 8086 binary file expected");
        eprintln!("USAGE: {} [8086 machine code file] \n", name);
        return;
    }
    for path in files {
        let program = fs::read(path).expect("filepath exists");

        println!(";;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;");
        println!(";;; disassembly of {}\n", path);

        let asm = disassemble(&program);
        print!("{}", asm);
    }
}
