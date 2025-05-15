use std::fmt::Write;

use crate::{Instruction, Mode, Place, RegisterByte, RegisterWord};

#[inline]
pub fn all_instructions(mut memory: &[u8]) -> String {
    let mut disassembly = String::from("bits 16\n");
    super::decode::all_instructions_into(&mut memory, &mut disassembly);
    disassembly
}

pub fn all_instructions_into(mut memory: &[u8], disassembly: &mut String) {
    while !memory.is_empty() {
        let (_, instruction) = single_instruction(&mut memory);
        disassembly
            .write_fmt(format_args!("{instruction}\n"))
            .expect("can write into the disassembly string");
        // disassembly.push_str(&single_instruction(&mut memory));
    }
}

pub fn single_instruction(memory: &mut &[u8]) -> (usize, Instruction) {
    let mem_left_prior = memory.len();
    let byte = advance(memory);

    let instruction = match byte {
        // ARITHMETIC Reg/memory wyth register to either
        0x00..=0x03 | 0x28..=0x2B | 0x38..=0x3B => {
            let word_flag = (byte & 0b01) > 0;
            let dest_flag = (byte & 0b10) > 0;
            let op = (byte >> 3) & 0b111;
            let (target, source) = target_source(word_flag, dest_flag, memory);
            Instruction::Arithmetic { op, target, source }
        }
        // ADD Immediate to accumulator
        0x04..=0x05 | 0x2C..=0x2D | 0x3C..=0x3D => {
            let word_mode = (byte & 0x01) > 0;
            let immediate = advance_by(memory, 1 + word_mode as usize) as u16;
            let op = (byte >> 3) & 0b111;
            let register = if word_mode {
                Place::Word(RegisterWord::AX)
            } else {
                Place::Byte(RegisterByte::AL)
            };
            Instruction::ArithmeticImmediate {
                op,
                target: register,
                immediate,
            }
        }
        0x70..=0x7F => {
            let offset = advance(memory) as i8;
            Instruction::Jump {
                marker: byte,
                offset,
            }
        }
        // LOOPS
        0xE0..=0xE3 => {
            let offset = advance(memory) as i8;
            Instruction::Loop {
                marker: byte,
                offset,
            }
        }
        // ARITHMETIC Immediate to register/memory
        0x80..=0x83 => {
            let word_mode = (byte & 0b01) > 0;

            // TODO (matyas): It's called sign extension because it's used when working with a 16 bit register
            // but there is only an 8 bit immediate that is negative, and the sign bit needs to move
            // from being the topmost bit of the 8 bits to being the topmost bit of the 16 bits.
            let sign_extension = (byte & 0b10) > 0;

            let (mode, op, r_m) = mod_reg_rm(memory);
            let mode = Mode::from_u8_discriminant(mode).unwrap();
            let target = Place::resolve_rm(r_m, mode, word_mode, memory);

            let immediate =
                advance_by(memory, 1 + ((word_mode && !sign_extension) as usize)) as u16;

            Instruction::ArithmeticImmediateToMemory {
                word_mode,
                op,
                target,
                immediate,
            }
        }
        // MOV Register/memory to/from register
        0x88..=0x8C => {
            let word_mode = (byte & 0b01) > 0;
            let dest_mode = (byte & 0b10) > 0;

            let (target, source) = target_source(word_mode, dest_mode, memory);
            Instruction::Mov { target, source }
        }
        // MOV Immediate
        0xB0..=0xBF => {
            let word_mode = (byte & 0b1000) > 0;
            let reg = byte & MASK_REG;

            let target = Place::register(word_mode, reg);
            let immediate = advance_by(memory, 1 + word_mode as usize) as u16;
            Instruction::MovImmediate { target, immediate }
        }
        0xC6 | 0xC7 => {
            let word_mode = (byte & 0b01) > 0;
            let (mode, _, r_m) = mod_reg_rm(memory);
            let mode = Mode::from_u8_discriminant(mode).unwrap();
            let target = Place::resolve_rm(r_m, mode, word_mode, memory);
            let immediate = advance_by(memory, 1 + word_mode as usize) as u16;
            Instruction::MovImmediateToMemory {
                word_mode,
                target,
                immediate,
            }
        }
        _ => Instruction::Unrecognized(byte),
    };
    let mem_left_after = memory.len();
    let offset = mem_left_prior - mem_left_after;

    (offset, instruction)
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

pub fn advance_by(memory: &mut &[u8], width: usize) -> usize {
    let displacement_bytes;
    (displacement_bytes, *memory) = memory.split_at(width);
    displacement_bytes
        .iter()
        .rev()
        .fold(0usize, |acc, byte| 256 * acc + *byte as usize)
}

const MASK_REG: u8 = 0b00000111;
