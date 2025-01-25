use std::{env::args, io::Read};

fn main() {
    let mut args = args();

    let Some(binary_file_path) = args.nth(1) else {
        eprintln!("no program to operate on");
        return;
    };

    let Ok(mut file) = std::fs::File::open(&binary_file_path) else {
        return;
    };

    let mut memory = Vec::new();
    let program_length = match file.read_to_end(&mut memory) {
        Err(error) => {
            eprintln!("Failed to read the file with error: {error}");
            return;
        }
        Ok(bytes_read) => bytes_read,
    };

    println!(";; dissassembly of {binary_file_path}\nbits 16\n");
    let mut memory = &memory[..program_length];
    while !memory.is_empty() {
        let byte = memory[0];
        memory = &memory[1..];

        let word_mode = (byte & MASK_W_BIT) != 0;
        let opcode = byte & MASK_OPCODE;
        match opcode {
            opcode::MOV => {
                print!("mov ");
                let mut _md = memory[0];
                let rm = _md & MASK_REG;
                _md >>= 3;
                let rg = _md & MASK_REG;
                _md >>= 3;

                let target = rm;
                let source = rg;

                if word_mode {
                    println!(
                        "{:?}, {:?}",
                        RegisterWord::from_u8_discriminant(target).unwrap(),
                        RegisterWord::from_u8_discriminant(source).unwrap(),
                    )
                } else {
                    println!(
                        "{:?}, {:?}",
                        RegisterByte::from_u8_discriminant(target).unwrap(),
                        RegisterByte::from_u8_discriminant(source).unwrap(),
                    )
                }
            }
            _ => println!("unrecognized opcode"),
        }
        memory = &memory[1..];
    }
}

const MASK_OPCODE: u8 = 0b11111100;
const MASK_D_BIT: u8 = 0b00000010;
const MASK_W_BIT: u8 = 0b00000001;

const MASK_REG: u8 = 0b00000111;

mod opcode {
    pub const MOV: u8 = 136;
}

#[derive(Debug)]
#[rustfmt::skip]
#[repr(u8)]
enum RegisterByte {
    AL, CL, DL, BL,
    AH, CH, DH, BH,
}

impl RegisterByte {
    fn from_u8_discriminant(discriminant: u8) -> Option<Self> {
        if discriminant > 0b111 {
            return None;
        }
        Some(unsafe { std::mem::transmute::<u8, RegisterByte>(discriminant) })
    }
}

#[derive(Debug)]
#[rustfmt::skip]
#[repr(u8)]
enum RegisterWord {
    AX, CX, DX, BX,
    SP, BP, SI, DI,
}

impl RegisterWord {
    fn from_u8_discriminant(discriminant: u8) -> Option<Self> {
        if discriminant > 0b111 {
            return None;
        }
        Some(unsafe { std::mem::transmute::<u8, RegisterWord>(discriminant) })
    }
}
