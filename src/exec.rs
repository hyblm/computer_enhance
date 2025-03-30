use std::{
    io::Read,
    ops::{Index, IndexMut},
};

use crate::{decode, Place, RegisterByte, RegisterWord};

pub fn all_instructions_and_print(state: &mut State) {
    while state.instruction_pointer < state.program_end {
        let (offset, instruction) =
            decode::single_instruction(&mut &state.memory[state.instruction_pointer..]);
        print!("; IP: {}\n{instruction}", state.instruction_pointer);
        state.instruction_pointer += offset;

        let flags_prior = state.registers.flags_string();
        if let Some(target) = instruction.target() {
            // TODO: generalize this for all cases. All that is needed is a function for getting
            // the value from state indexed by Place
            match target {
                Place::Byte(reg) => {
                    print!("; {target}: {:x} -> ", state.registers[reg]);
                    instruction.run(state);
                    print!("{:x}", state.registers[reg]);
                }
                Place::Word(reg) => {
                    print!("; {target}: {:x} -> ", state.registers[reg]);
                    instruction.run(state);
                    print!("{:x}", state.registers[reg]);
                }
                Place::Adress(address) => match address.mode {
                    crate::Mode::EffectiveAdress => match address.index {
                        0b110 => {
                            let hi = state.memory[address.displacement as usize];
                            let lo = state.memory[(address.displacement + 1) as usize];
                            let value = u16::from_be_bytes([hi, lo]);
                            print!("; {target}: {value:x} -> ");
                            instruction.run(state);
                            let hi = state.memory[address.displacement as usize];
                            let lo = state.memory[(address.displacement + 1) as usize];
                            let value = u16::from_be_bytes([hi, lo]);
                            print!("{value:x}");
                        }
                        0b010 => {
                            let i = (state.registers[RegisterWord::BP]
                                + state.registers[RegisterWord::SI]
                                + address.displacement)
                                as usize;
                            let hi = state.memory[i];
                            let lo = state.memory[i + 1];
                            let value = u16::from_be_bytes([hi, lo]);
                            print!("; {target}: {value:x} -> ");
                            instruction.run(state);
                            let hi = state.memory[i];
                            let lo = state.memory[i + 1];
                            let value = u16::from_be_bytes([hi, lo]);
                            print!("{value:x}");
                        }
                        _ => todo!(),
                    },
                    crate::Mode::EffectiveAdressByte => {
                        let i: usize = (match address.index {
                            0b111 => state.registers[RegisterWord::BX],
                            0b110 => state.registers[RegisterWord::BP],
                            _ => todo!(),
                        } + address.displacement)
                            .into();
                        let hi = state.memory[i];
                        let lo = state.memory[i + 1];
                        let value = u16::from_be_bytes([hi, lo]);
                        print!("; {target}: {value:x} -> ");
                        instruction.run(state);
                        let hi = state.memory[i];
                        let lo = state.memory[i + 1];
                        let value = u16::from_be_bytes([hi, lo]);
                        print!("{value:x}");
                    }
                    crate::Mode::EffectiveAdressWord => todo!(),
                    crate::Mode::RegisterToRegister => todo!(),
                },
            }
        } else {
            instruction.run(state);
        };
        let flags_after = state.registers.flags_string();
        if flags_after != flags_prior {
            print!(" flags: {flags_prior} -> {flags_after}");
        }
        println!();
    }
}

union SplitRegister {
    byte: SplitRegisterInner,
    word: u16,
}
impl Default for SplitRegister {
    fn default() -> Self {
        Self { word: 0 }
    }
}

#[derive(Clone, Copy, Default)]
struct SplitRegisterInner {
    h: u8,
    l: u8,
}

pub struct State {
    pub registers: Registers,
    pub memory: [u8; u16::MAX as usize],
    program_end: usize,
    pub instruction_pointer: usize,
}

impl Default for State {
    fn default() -> Self {
        Self {
            memory: [0; u16::MAX as usize],
            registers: Registers::default(),
            program_end: 0,
            instruction_pointer: 0,
        }
    }
}
impl State {
    pub fn load_program(&mut self, path_bin: &str) {
        let mut file = std::fs::File::open(path_bin).unwrap();

        let mut memory = Vec::with_capacity(u16::MAX as usize);
        self.program_end = match file.read_to_end(&mut memory) {
            Err(error) => {
                panic!("Failed to read the file with error: {error}");
            }
            Ok(bytes_read) => bytes_read,
        };
        self.memory[..memory.len()].copy_from_slice(memory.as_slice());
    }
}

// TODO (matyas): add segment registers
// TODO (matyas): add the rest of the flags
#[derive(Default)]
pub struct Registers {
    data_group: [SplitRegister; 4],
    meta_group: [u16; 4],
    pub flag_zero: bool,
    pub flag_sign: bool,
}

impl Registers {
    pub fn flags_string(&self) -> String {
        format!(
            "{}{}",
            if self.flag_sign { "S" } else { "" },
            if self.flag_zero { "Z" } else { "" }
        )
    }
    pub fn print(&self) {
        use RegisterWord::*;
        println!(
            "AX: {}\nBX: {}\nCX: {}\nDX: {}\nSP: {}\nBP: {}\nSI: {}\nDI: {}",
            self[AX], self[BX], self[CX], self[DX], self[SP], self[BP], self[SI], self[DI]
        )
    }
}

impl Index<RegisterWord> for Registers {
    type Output = u16;

    fn index(&self, index: RegisterWord) -> &Self::Output {
        match index {
            RegisterWord::AX => unsafe { &self.data_group[0].word },
            RegisterWord::CX => unsafe { &self.data_group[1].word },
            RegisterWord::DX => unsafe { &self.data_group[2].word },
            RegisterWord::BX => unsafe { &self.data_group[3].word },
            RegisterWord::SP => &self.meta_group[0],
            RegisterWord::BP => &self.meta_group[1],
            RegisterWord::SI => &self.meta_group[2],
            RegisterWord::DI => &self.meta_group[3],
        }
    }
}

impl IndexMut<RegisterWord> for Registers {
    fn index_mut(&mut self, index: RegisterWord) -> &mut Self::Output {
        match index {
            RegisterWord::AX => unsafe { &mut self.data_group[0].word },
            RegisterWord::CX => unsafe { &mut self.data_group[1].word },
            RegisterWord::DX => unsafe { &mut self.data_group[2].word },
            RegisterWord::BX => unsafe { &mut self.data_group[3].word },
            RegisterWord::SP => &mut self.meta_group[0],
            RegisterWord::BP => &mut self.meta_group[1],
            RegisterWord::SI => &mut self.meta_group[2],
            RegisterWord::DI => &mut self.meta_group[3],
        }
    }
}

impl Index<RegisterByte> for Registers {
    type Output = u8;

    fn index(&self, index: RegisterByte) -> &Self::Output {
        match index {
            RegisterByte::AH => unsafe { &self.data_group[0].byte.h },
            RegisterByte::AL => unsafe { &self.data_group[0].byte.l },
            RegisterByte::BH => unsafe { &self.data_group[1].byte.h },
            RegisterByte::BL => unsafe { &self.data_group[1].byte.l },
            RegisterByte::CH => unsafe { &self.data_group[2].byte.h },
            RegisterByte::CL => unsafe { &self.data_group[2].byte.l },
            RegisterByte::DH => unsafe { &self.data_group[3].byte.h },
            RegisterByte::DL => unsafe { &self.data_group[3].byte.l },
        }
    }
}
impl IndexMut<RegisterByte> for Registers {
    fn index_mut(&mut self, index: RegisterByte) -> &mut Self::Output {
        match index {
            RegisterByte::AH => unsafe { &mut self.data_group[0].byte.h },
            RegisterByte::AL => unsafe { &mut self.data_group[0].byte.l },
            RegisterByte::BH => unsafe { &mut self.data_group[1].byte.h },
            RegisterByte::BL => unsafe { &mut self.data_group[1].byte.l },
            RegisterByte::CH => unsafe { &mut self.data_group[2].byte.h },
            RegisterByte::CL => unsafe { &mut self.data_group[2].byte.l },
            RegisterByte::DH => unsafe { &mut self.data_group[3].byte.h },
            RegisterByte::DL => unsafe { &mut self.data_group[3].byte.l },
        }
    }
}
