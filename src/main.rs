use std::{env::args, io::Read};

#[cfg(test)]
mod tests;

fn main() {
    let binary_file_path = args().nth(1).unwrap();
    let mut file = std::fs::File::open(binary_file_path).unwrap();

    let mut memory = Vec::new();
    let program_length = match file.read_to_end(&mut memory) {
        Err(error) => {
            panic!("Failed to read the file with error: {error}");
        }
        Ok(bytes_read) => bytes_read,
    };

    let mut disassembly = String::from(";; dissassembly of {binary_file_path}\nbits 16\n");
    decode_program_instructions(&memory[..program_length], &mut disassembly);
}

fn decode_program_instructions(mut memory: &[u8], disassembly: &mut String) {
    while !memory.is_empty() {
        decode_instruction(&mut memory, disassembly);
    }
    println!("{disassembly}");
}

fn decode_instruction(memory: &mut &[u8], disassembly: &mut String) {
    let result;
    let byte = advance(memory);
    let nibble = byte >> 4;
    let opcode = byte & MASK_OPCODE;

    if nibble == opcode::MOV_IMMEDIATE {
        let word_mode = (byte & 0b1000) > 0;
        let reg = byte & MASK_REG;

        result = if word_mode {
            let data_bytes;
            (data_bytes, *memory) = memory.split_at(2);
            format!(
                "mov {:?}, {}\n",
                RegisterWord::from_u8_discriminant(reg).unwrap(),
                u16::from_le_bytes([data_bytes[0], data_bytes[1]])
            )
        } else {
            let data = advance(memory);
            format!(
                "mov {:?}, {}\n",
                RegisterByte::from_u8_discriminant(reg).unwrap(),
                data
            )
        }
    } else if opcode == opcode::MOV {
        let mut mode = advance(memory);
        let rm = mode & MASK_REG;
        mode >>= 3;
        let reg = mode & MASK_REG;
        let reg_is_target = (byte & MASK_D_BIT) != 0;
        mode >>= 3;
        let word_mode = (byte & MASK_W_BIT) != 0;
        result = if let Mode::RegisterToRegister = Mode::from_u8_discriminant(mode).unwrap() {
            if word_mode {
                let t = RegisterWord::from_u8_discriminant(rm).unwrap();
                let s = RegisterWord::from_u8_discriminant(reg).unwrap();
                format!("mov {:?}, {:?}\n", t, s)
            } else {
                let t = RegisterByte::from_u8_discriminant(rm).unwrap();
                let s = RegisterByte::from_u8_discriminant(reg).unwrap();
                format!("mov {:?}, {:?}\n", t, s)
            }
        } else {
            let register = if word_mode {
                format!("{:?}", RegisterWord::from_u8_discriminant(reg).unwrap())
            } else {
                format!("{:?}", RegisterByte::from_u8_discriminant(reg).unwrap())
            };

            let displacement_bytes;
            (displacement_bytes, *memory) = memory.split_at(mode as usize);
            let displacement = displacement_bytes
                .iter()
                .rev()
                .fold(0usize, |acc, byte| 256 * acc + *byte as usize);
            let adress_calculation =
                format!("[{} + {}]", ADRESS_CALCULATION[rm as usize], displacement);

            if reg_is_target {
                format!("mov {}, {}\n", register, adress_calculation)
            } else {
                format!("mov {}, {}\n", adress_calculation, register)
            }
        };
    } else {
        result = format!("unrecognized opcode in {byte:3} {byte:2x} {byte:8b}\n");
    }
    disassembly.push_str(&result);
}

fn advance(memory: &mut &[u8]) -> u8 {
    let byte = memory[0];
    *memory = &memory[1..];
    byte
}

const MASK_OPCODE: u8 = 0b11111100;
const MASK_D_BIT: u8 = 0b00000010;
const MASK_W_BIT: u8 = 0b00000001;

const MASK_REG: u8 = 0b00000111;

const ADRESS_CALCULATION: [&str; 8] = [
    "BX + SI", "BX + DI", "BP + SI", "BP + DI", "SI", "DI", "BP", "BX",
];

mod opcode {
    pub const MOV_IMMEDIATE: u8 = 0xB;
    pub const MOV: u8 = 136;
}

#[repr(u8)]
enum Mode {
    EffectiveAdress = 0b00,
    EffectiveAdressByte = 0b01,
    EffectiveAdressWord = 0b10,
    RegisterToRegister = 0b11,
}

impl Mode {
    fn from_u8_discriminant(discriminant: u8) -> Option<Self> {
        if discriminant > 0b11 {
            return None;
        }
        Some(unsafe { std::mem::transmute::<u8, Self>(discriminant) })
    }
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
        Some(unsafe { std::mem::transmute::<u8, Self>(discriminant) })
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
        Some(unsafe { std::mem::transmute::<u8, Self>(discriminant) })
    }
}
