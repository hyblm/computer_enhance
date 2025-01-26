use std::io::{Read, Write};

use crate::decode_program_instructions;

fn process_file_listing(binary_file_path: &str) {
    let (memory, program_length) = read_listing(binary_file_path);

    let mut disassembly = String::new();
    decode_program_instructions(&memory[..program_length], &mut disassembly);

    let path_binary = format!("{binary_file_path}_generated");
    let path_asm = format!("{path_binary}.asm");
    let mut file = std::fs::File::create(&path_asm).unwrap();

    file.write_all(disassembly.as_bytes()).unwrap();
    std::process::Command::new("nasm")
        .arg(&path_asm)
        .output()
        .expect("failed to execute process");

    let (new_memory, new_program_length) = read_listing(&path_binary);
    std::fs::remove_file(&path_asm).unwrap();
    std::fs::remove_file(&path_binary).unwrap();

    assert_eq!(program_length, new_program_length);
    assert_eq!(memory, new_memory);
}

fn read_listing(binary_file_path: &str) -> (Vec<u8>, usize) {
    let mut file = std::fs::File::open(binary_file_path).unwrap();

    let mut memory = Vec::new();
    let program_length = match file.read_to_end(&mut memory) {
        Err(error) => {
            panic!("Failed to read the file with error: {error}");
        }
        Ok(bytes_read) => bytes_read,
    };

    (memory, program_length)
}

mod base {
    use super::process_file_listing;

    #[test]
    fn listing_37() {
        process_file_listing("part_1/listing_0037_single_register_mov");
    }

    #[test]
    fn listing_38() {
        process_file_listing("part_1/listing_0038_many_register_mov");
    }

    #[test]
    fn listing_39() {
        process_file_listing("part_1/listing_0039_more_movs");
    }
}

mod challenge {
    use super::process_file_listing;

    #[test]
    fn listing_40() {
        process_file_listing("part_1/listing_0040_challenge_movs");
    }
}
