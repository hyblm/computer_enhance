use std::io::Write;

use crate::{decode, read_listing};

fn process_file_listing(binary_file_path: &str) {
    // disassemble listing
    let (memory, length) = read_listing(binary_file_path);
    let disassembly = decode::all_instructions(&memory[..length]);
    let (memory_new, length_new) = assemble(disassembly, binary_file_path);

    assert_eq!(length, length_new);
    assert_eq!(memory, memory_new);
}

/// Save the dissasembly into a file, run nasm on it, and then load and return the resulting binary.
/// The assembly and binary files are deleted before this function returns.
fn assemble(disassembly: String, base_name: &str) -> (Vec<u8>, usize) {
    let path_bin = format!("{base_name}_generated");
    let path_asm = format!("{path_bin}.asm");

    // save disassembly into a file and run it through nasm
    let mut file = std::fs::File::create(&path_asm).unwrap();
    file.write_all(disassembly.as_bytes()).unwrap();
    std::process::Command::new("nasm")
        .arg(&path_asm)
        .output()
        .expect("failed to execute process");

    eprintln!("{path_bin}");
    eprintln!("{path_asm}");
    let (new_memory, new_program_length) = read_listing(&path_bin);

    // cleanup before asserts
    std::fs::remove_file(&path_asm).unwrap();
    std::fs::remove_file(&path_bin).unwrap();
    (new_memory, new_program_length)
}

mod decoding {
    use crate::tests::process_file_listing;

    #[test]
    fn listing_37_single_register_mov() {
        process_file_listing("part_1/listing_0037_single_register_mov");
    }

    #[test]
    fn listing_38_many_register_mov() {
        process_file_listing("part_1/listing_0038_many_register_mov");
    }

    #[test]
    fn listing_39_more_movs() {
        process_file_listing("part_1/listing_0039_more_movs");
    }

    #[test]
    fn listing_41_add_sub_cmp_jnz() {
        process_file_listing("part_1/listing_0041_add_sub_cmp_jnz");
    }

    #[test]
    fn listing_43_immediate_movs() {
        process_file_listing("part_1/listing_0043_immediate_movs");
    }

    #[test]
    fn listing_44_register_movs() {
        process_file_listing("part_1/listing_0044_register_movs");
    }
}

mod simulation {
    #[test]
    fn listing_43_immediate_movs() {}

    #[test]
    fn listing_44_register_movs() {}
}

mod challenge {
    // use super::process_file_listing;

    // #[test]
    // fn listing_40() {
    //     process_file_listing("part_1/listing_0040_challenge_movs");
    // }

    // #[test]
    // fn listing_42() {
    //     process_file_listing("part_1/listing_0042_completionist_decode");
    // }

    // #[test]
    // fn listing_45() {
    //     process_file_listing("part_1/listing_0045_challenge_register_movs");
    // }
}
