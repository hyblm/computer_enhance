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
        disassembly.push_str(&decode_instruction(&mut memory));
    }
    println!("{disassembly}");
}

fn decode_instruction(memory: &mut &[u8]) -> String {
    let byte = advance(memory);

    match byte {
        // ARITHMETIC Reg/memory wyth register to either
        0x00..=0x03 | 0x28..=0x2B | 0x38..=0x3B => {
            let word_mode = (byte & 0b01) > 0;
            let dest_mode = (byte & 0b10) > 0;
            let op = (byte >> 3) & 0b111;
            let op = ARITHMETIC[op as usize];
            let (target, source) = target_source(word_mode, dest_mode, memory);

            format!("{op} {target}, {source}\n")
        }
        // ADD Immediate to accumulator
        0x04..=0x05 | 0x2C..=0x2D | 0x3C..=0x3D => {
            let word_mode = (byte & 0x01) > 0;
            let immediate = advance_by(memory, 1 + word_mode as usize);
            let op = (byte >> 3) & 0b111;
            let op = ARITHMETIC[op as usize];
            let register = if word_mode {
                Place::Word(RegisterWord::AX)
            } else {
                Place::Byte(RegisterByte::AL)
            };
            format!("{op} {register}, {immediate}\n",)
        }
        0x70..=0x7F => {
            let negated = if (byte & 1) > 0 { "n" } else { "" };
            let kind = JUMP[((byte >> 1) & 0b111) as usize];
            let offset = 2 + advance(memory) as i8;
            format!("j{negated}{kind} ${offset:+}\n",)
        }
        // LOOPS
        0xE0..=0xE3 => {
            let kind = LOOP[(byte & 0b11) as usize];
            let displacement = 2 + advance(memory) as i8;
            format!("{kind} ${displacement:+}\n",)
        }
        // ARITHMETIC Immediate to register/memory
        0x80..=0x83 => {
            let word_mode = (byte & 0b01) > 0;
            let sign_extension = (byte & 0b10) > 0;
            let (mode, op, r_m) = mod_reg_rm(memory);
            let mode = Mode::from_u8_discriminant(mode).unwrap();
            let r_m = Place::resolve_rm(r_m, mode, word_mode, memory);
            let immediate = advance(memory);
            let size = if word_mode { "word" } else { "byte" };

            format!("{} {size} {r_m}, {immediate}\n", ARITHMETIC[op as usize])
        }
        // MOV Register/memory to/from register
        0x88..=0x8C => {
            let word_mode = (byte & 0b01) > 0;
            let dest_mode = (byte & 0b10) > 0;

            let (target, source) = target_source(word_mode, dest_mode, memory);
            format!("mov {target}, {source}\n")
        }
        // MOV Immediate
        0xB0..=0xBF => {
            let word_mode = (byte & 0b1000) > 0;
            let reg = byte & MASK_REG;

            let register = Place::register(word_mode, reg);
            let data = advance_by(memory, 1 + word_mode as usize);
            format!("mov {register}, {data}\n")
        }
        0xC6 | 0xC7 => {
            let word_mode = (byte & 0b01) > 0;
            let size = if word_mode { "word" } else { "byte" };
            let (mode, op, r_m) = mod_reg_rm(memory);
            let mode = Mode::from_u8_discriminant(mode).unwrap();
            let r_m = Place::resolve_rm(r_m, mode, word_mode, memory);
            let immediate = advance(memory);
            format!("mov {r_m}, {size} {immediate} \n")
        }
        _ => {
            format!("\x1B[1m\x1B[31m{byte:2x}\x1B[0m unrecognized\n")
        }
    }
}

fn target_source(word_mode: bool, dest_mode: bool, memory: &mut &[u8]) -> (Place, Place) {
    let (mode, reg, r_m) = mod_reg_rm(memory);
    let mode = Mode::from_u8_discriminant(mode).unwrap();
    let reg = Place::register(word_mode, reg);
    let r_m = Place::resolve_rm(r_m, mode, word_mode, memory);
    if dest_mode {
        (reg, r_m)
    } else {
        (r_m, reg)
    }
}

fn mod_reg_rm(memory: &mut &[u8]) -> (u8, u8, u8) {
    let mut byte = advance(memory);
    let r_m = byte & MASK_REG;
    byte >>= 3;
    let reg = byte & MASK_REG;
    byte >>= 3;
    (byte, reg, r_m)
}

fn advance(memory: &mut &[u8]) -> u8 {
    if memory.is_empty() {
        return 0;
    }
    let byte = memory[0];
    *memory = &memory[1..];
    byte
}

fn advance_by(memory: &mut &[u8], width: usize) -> usize {
    let displacement_bytes;
    (displacement_bytes, *memory) = memory.split_at(width);
    displacement_bytes
        .iter()
        .rev()
        .fold(0usize, |acc, byte| 256 * acc + *byte as usize)
}

const MASK_REG: u8 = 0b00000111;
const ARITHMETIC: [&str; 8] = ["add", "or", "adc", "sbb", "and", "sub", "xor", "cmp"];
const JUMP: [&str; 8] = ["o", "b", "z", "be", "s", "p", "l", "le"];
const LOOP: [&str; 4] = ["loopnz", "loopz", "loop", "jcxz"];

#[derive(Debug)]
struct EffectiveAdress {
    index: u8,
    mode: Mode,
    displacement: u16,
}
impl EffectiveAdress {
    const ADRESS_CALCULATION: [&'static str; 8] = [
        "BX + SI", "BX + DI", "BP + SI", "BP + DI", "SI", "DI", "BP", "BX",
    ];
}
impl std::fmt::Display for EffectiveAdress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.mode == Mode::EffectiveAdress && self.index == 0b110 {
            write!(f, "[{}]", self.displacement)
        } else {
            write!(
                f,
                "[{} + {}]",
                Self::ADRESS_CALCULATION[self.index as usize],
                self.displacement
            )
        }
    }
}

#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
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
enum Place {
    Byte(RegisterByte),
    Word(RegisterWord),
    Adress(EffectiveAdress),
}
impl std::fmt::Display for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Place::Byte(reg) => write!(f, "{reg:?}"),
            Place::Word(reg) => write!(f, "{reg:?}"),
            Place::Adress(ea) => ea.fmt(f),
        }
    }
}

impl Place {
    fn register(word_mode: bool, disc: u8) -> Self {
        if word_mode {
            Self::Word(RegisterWord::from_octal(disc).unwrap())
        } else {
            Self::Byte(RegisterByte::from_octal(disc).unwrap())
        }
    }

    fn adress(r_m: u8, mode: Mode, memory: &mut &[u8]) -> Self {
        let displacement = if mode == Mode::EffectiveAdress && r_m == 0b110 {
            advance_by(memory, 2)
        } else {
            advance_by(memory, mode as usize)
        } as u16;
        Self::Adress(EffectiveAdress {
            index: r_m,
            mode,
            displacement,
        })
    }

    fn resolve_rm(r_m: u8, mode: Mode, word_mode: bool, memory: &mut &[u8]) -> Self {
        if let Mode::RegisterToRegister = mode {
            Place::register(word_mode, r_m)
        } else {
            Place::adress(r_m, mode, memory)
        }
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
    fn from_octal(octal: u8) -> Option<Self> {
        if octal > 0b111 {
            return None;
        }
        Some(unsafe { std::mem::transmute::<u8, Self>(octal) })
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
    fn from_octal(octal: u8) -> Option<Self> {
        if octal > 0b111 {
            return None;
        }
        Some(unsafe { std::mem::transmute::<u8, Self>(octal) })
    }
}
