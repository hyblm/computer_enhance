use std::{env::args, fs::File, io::Write};

use sim86::{
    decode,
    exec::{self, State},
    read_listing,
};

fn main() {
    let args = &mut args();
    let _ = args.next();
    let mut dump = false;

    let mut state = State::default();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-dump" => {
                dump = true;
                continue;
            }
            "-exec" => {
                let Some(path_bin) = args.next() else {
                    eprintln!("no binary provided");
                    return;
                };

                state.load_program(&path_bin);
                // let (memory, _length) = read_listing(&path_bin);
                // let mut registers = Registers::default();
                exec::all_instructions_and_print(&mut state);
                state.registers.print();
            }
            _ => {
                let path_bin = arg;
                let (memory, length) = read_listing(&path_bin);
                let disassembly = decode::all_instructions(&memory[..length]);
                println!("{disassembly}");
            }
        }
    }

    if dump {
        let mut count = 0;
        loop {
            if let Ok(mut file) = File::create_new(format!("sim8086_memory_{count}.data")) {
                match file.write_all(&state.memory) {
                    Ok(_) => (),
                    Err(error) => eprintln!("failed to dump memory to file with error: {error}"),
                }
                break;
            } else {
                count += 1;
            }
        }
    }
}
