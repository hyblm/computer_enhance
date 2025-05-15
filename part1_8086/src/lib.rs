use std::{fmt::Display, io::Read};

use decode::advance_by;
use exec::State;

pub mod decode;
pub mod exec;

#[cfg(test)]
mod tests;

pub fn read_listing(listing_path: &str) -> (Vec<u8>, usize) {
    let mut file = std::fs::File::open(listing_path).unwrap();

    let mut memory = Vec::new();
    let length = match file.read_to_end(&mut memory) {
        Err(error) => {
            panic!("Failed to read the file with error: {error}");
        }
        Ok(bytes_read) => bytes_read,
    };

    (memory, length)
}

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    Arithmetic {
        op: u8,
        target: Place,
        source: Place,
    },
    ArithmeticImmediate {
        op: u8,
        target: Place,
        immediate: u16,
    },
    ArithmeticImmediateToMemory {
        word_mode: bool,
        op: u8,
        target: Place,
        immediate: u16,
    },
    Jump {
        marker: u8,
        offset: i8,
    },
    Loop {
        marker: u8,
        offset: i8,
    },
    Mov {
        target: Place,
        source: Place,
    },
    MovImmediate {
        target: Place,
        immediate: u16,
    },
    MovImmediateToMemory {
        word_mode: bool,
        target: Place,
        immediate: u16,
    },
    Unrecognized(u8),
}
impl Instruction {
    fn target(&self) -> Option<Place> {
        match self {
            Instruction::Arithmetic { target, .. }
            | Instruction::ArithmeticImmediate { target, .. }
            | Instruction::ArithmeticImmediateToMemory { target, .. }
            | Instruction::Mov { target, .. }
            | Instruction::MovImmediate { target, .. }
            | Instruction::MovImmediateToMemory { target, .. } => Some(*target),
            _ => None,
        }
    }

    pub const MATH: [&'static str; 8] = ["add", "or", "adc", "sbb", "and", "sub", "xor", "cmp"];
    pub const JUMP: [&'static str; 8] = ["o", "b", "z", "be", "s", "p", "l", "le"];
    pub const LOOP: [&'static str; 4] = ["loopnz", "loopz", "loop", "jcxz"];

    #[allow(unused_variables)]
    fn run(self, state: &mut State) {
        use RegisterWord::*;
        match self {
            Instruction::MovImmediate { target, immediate } => match target {
                Place::Byte(reg) => state.registers[reg] = immediate as u8,
                Place::Word(reg) => state.registers[reg] = immediate,
                Place::Adress(_) => todo!(),
            },
            Instruction::Arithmetic { op, target, source } => {
                let result = match (Self::MATH[op as usize], target, source) {
                    // (Place::Byte(_), Place::Byte(_)) => todo!(),
                    // (Place::Byte(_), Place::Word(_)) => todo!(),
                    // (Place::Byte(_), Place::Adress(_)) => todo!(),
                    // (Place::Word(_), Place::Byte(_)) => todo!(),
                    ("add", Place::Word(target), Place::Word(source)) => {
                        state.registers[target] =
                            state.registers[target].wrapping_add(state.registers[source]);
                        state.registers[target]
                    }
                    ("add", Place::Word(target), Place::Adress(source)) => {
                        match source.mode {
                            Mode::EffectiveAdress => {
                                let i: usize = match source.index {
                                    0b110 => source.displacement,
                                    0b010 => state.registers[BP] + state.registers[SI],
                                    _ => todo!(),
                                }
                                .into();
                                let hi = state.memory[i];
                                let lo = state.memory[i + 1];
                                let value = u16::from_be_bytes([hi, lo]);
                                state.registers[target] = value;
                            }
                            Mode::EffectiveAdressByte => todo!(),
                            Mode::EffectiveAdressWord => todo!(),
                            Mode::RegisterToRegister => todo!(),
                        };
                        // registers[target] = registers[target].wrapping_add(registers[source]);
                        state.registers[target]
                    }
                    ("sub", Place::Word(target), Place::Word(source)) => {
                        state.registers[target] =
                            state.registers[target].wrapping_sub(state.registers[source]);
                        state.registers[target]
                    }
                    ("cmp", Place::Word(target), Place::Word(source)) => {
                        state.registers[target].wrapping_sub(state.registers[source])
                    }
                    // (Place::Word(_), Place::Adress(_)) => todo!(),
                    // (Place::Adress(_), Place::Byte(_)) => todo!(),
                    // (Place::Adress(_), Place::Word(_)) => todo!(),
                    // (Place::Adress(_), Place::Adress(_)) => todo!(),
                    _ => todo!(),
                };
                state.registers.flag_sign = (result & 0x8000) > 0;
                state.registers.flag_zero = result == 0;
            }
            Instruction::ArithmeticImmediate {
                op,
                target: register,
                immediate,
            } => todo!(),
            Instruction::ArithmeticImmediateToMemory {
                word_mode,
                op,
                target,
                immediate,
            } => {
                let result = match (Self::MATH[op as usize], target) {
                    ("add", Place::Word(target)) => {
                        state.registers[target] = state.registers[target].wrapping_add(immediate);
                        state.registers[target]
                    }
                    ("sub", Place::Word(target)) => {
                        state.registers[target] = state.registers[target].wrapping_sub(immediate);
                        state.registers[target]
                    }
                    ("cmp", Place::Word(target)) => state.registers[target].wrapping_sub(immediate),
                    _ => todo!(),
                };
                state.registers.flag_sign = (result & 0x8000) > 0;
                state.registers.flag_zero = result == 0;
            }
            Instruction::Jump { marker, offset } => {
                let negated = (marker & 1) > 0;
                let kind = Self::JUMP[((marker >> 1) & 0b111) as usize];
                match kind {
                    "o" => todo!(),
                    "b" => todo!(),
                    "z" => {
                        if state.registers.flag_zero ^ negated {
                            if offset.is_positive() {
                                state.instruction_pointer += offset as usize;
                            } else {
                                state.instruction_pointer -= offset.abs() as usize;
                            }
                        }
                    }
                    "be" => todo!(),
                    "s" => todo!(),
                    "p" => todo!(),
                    "l" => todo!(),
                    "le" => todo!(),
                    _ => unreachable!(),
                }
                // todo!()
            }
            Instruction::Loop { marker, offset } => todo!(),
            Instruction::Mov { target, source } => match (target, source) {
                (Place::Byte(target), Place::Byte(source)) => todo!(),
                (Place::Byte(target), Place::Word(source)) => todo!(),
                (Place::Byte(target), Place::Adress(source)) => todo!(),
                (Place::Word(target), Place::Byte(source)) => todo!(),
                (Place::Word(target), Place::Word(source)) => {
                    state.registers[target] = state.registers[source]
                }
                (Place::Word(target), Place::Adress(source)) => match source.mode {
                    Mode::EffectiveAdress => {
                        let i: usize = match source.index {
                            0b110 => source.displacement,
                            0b010 => state.registers[BP] + state.registers[SI],
                            _ => todo!(),
                        }
                        .into();
                        let hi = state.memory[i];
                        let lo = state.memory[i + 1];
                        let value = u16::from_be_bytes([hi, lo]);
                        state.registers[target] = value;
                    }
                    Mode::EffectiveAdressByte => todo!(),
                    Mode::EffectiveAdressWord => todo!(),
                    Mode::RegisterToRegister => todo!(),
                },
                (Place::Adress(target), Place::Byte(source)) => todo!(),
                (Place::Adress(target), Place::Word(source)) => {
                    let value = state.registers[source];
                    let bytes = value.to_be_bytes();
                    match target.mode {
                        Mode::EffectiveAdress => {
                            let i: usize = match target.index {
                                0b010 => state.registers[BP] + state.registers[SI],
                                _ => todo!(),
                            }
                            .into();
                            state.memory[i..(i + 2)].copy_from_slice(&bytes);
                        }
                        Mode::EffectiveAdressByte => {
                            let i: usize = match target.index {
                                0b110 => {
                                    // print!("      uwu      ");
                                    state.registers[BP]
                                }
                                _ => todo!(),
                            }
                            .into();
                            state.memory[i] = value as u8;
                        }
                        Mode::EffectiveAdressWord => todo!(),
                        Mode::RegisterToRegister => todo!(),
                    }
                }
                (Place::Adress(target), Place::Adress(source)) => todo!(),
            },
            Instruction::MovImmediateToMemory {
                word_mode,
                target,
                immediate,
            } => match target {
                Place::Byte(_) => todo!(),
                Place::Word(_) => todo!(),
                Place::Adress(address) => {
                    let bytes = immediate.to_be_bytes();
                    let i: usize = match address.mode {
                        Mode::EffectiveAdress => match address.index {
                            0b110 => address.displacement,
                            _ => todo!(),
                        },
                        Mode::EffectiveAdressByte => {
                            (match address.index {
                                0b110 => state.registers[BP],
                                0b111 => state.registers[BX],
                                _ => todo!(),
                            } + address.displacement)
                        }
                        Mode::EffectiveAdressWord => todo!(),
                        Mode::RegisterToRegister => todo!(),
                    }
                    .into();
                    state.memory[i..(i + 2)].copy_from_slice(&bytes);
                }
            },
            Instruction::Unrecognized(_) => todo!(),
        };
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Arithmetic { op, target, source } => {
                let var_name = {
                    let op = Self::MATH[*op as usize];
                    write!(f, "{op} {target}, {source}")
                };
                var_name
            }
            Instruction::ArithmeticImmediate {
                op,
                target: register,
                immediate,
            } => {
                let op = Self::MATH[*op as usize];
                write!(f, "{op} {register}, {immediate}")
            }
            Instruction::ArithmeticImmediateToMemory {
                word_mode,
                op,
                target,
                immediate,
            } => {
                let op = Self::MATH[*op as usize];
                let size = if *word_mode { "word" } else { "byte" };
                write!(f, "{op} {size} {target}, {immediate}")
            }
            Instruction::Jump { marker, offset } => {
                let negated = if (marker & 1) > 0 { "n" } else { "" };
                let kind = Self::JUMP[((marker >> 1) & 0b111) as usize];
                write!(f, "j{negated}{kind} $+2{offset:+}")
            }
            Instruction::Loop { marker, offset } => {
                let kind = Self::LOOP[(marker & 0b11) as usize];
                write!(f, "{kind} $+2{offset:+}")
            }
            Instruction::Mov { target, source } => write!(f, "mov {target}, {source}"),
            Instruction::MovImmediate { target, immediate } => {
                write!(f, "mov {target}, {immediate}")
            }
            Instruction::MovImmediateToMemory {
                word_mode,
                target,
                immediate,
            } => {
                let size = if *word_mode { "word" } else { "byte" };
                write!(f, "mov {target}, {size} {immediate}")
            }
            Instruction::Unrecognized(byte) => {
                write!(f, "\x1B[1m\x1B[31m{byte:2x}\x1B[0m unrecognized")
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EffectiveAdress {
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

#[allow(dead_code)]
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

#[derive(Debug, Clone, Copy)]
pub enum Place {
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

    fn address(r_m: u8, mode: Mode, memory: &mut &[u8]) -> Self {
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
            Place::address(r_m, mode, memory)
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
#[rustfmt::skip]
#[repr(u8)]
pub enum RegisterByte {
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

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
#[rustfmt::skip]
#[repr(u8)]
pub enum RegisterWord {
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
