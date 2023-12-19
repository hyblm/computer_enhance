use crate::disassemble;
use rand::{distributions::Alphanumeric, Rng};
use std::{
    fs::{self, remove_file},
    process::Command,
};

#[test]
fn listing37() {
    validate_listing("listing37")
}
#[test]
fn listing38() {
    validate_listing("listing38")
}
#[test]
fn listing39() {
    validate_listing("listing39")
}

mod listing39;

fn validate_asm(asm: &str) {
    println!("{}", asm);
    // assemble the listing
    let listing: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    let src_path = format!("{listing}.asm");
    let _ = fs::write(&src_path, asm);
    Command::new("nasm").args([&src_path]).output().unwrap();

    // disassemble
    let src = fs::read(&listing).expect("listing is in project directory");
    remove_file(&listing).unwrap();
    remove_file(src_path).unwrap();
    let dasm = disassemble(&src);

    let dasm_path = format!("{listing}-d.asm");
    let _ = fs::write(&dasm_path, dasm);

    // assemble the disassembly
    Command::new("nasm").args([&dasm_path]).output().unwrap();

    let bin_path = format!("{listing}-d");
    let new = fs::read(&bin_path).expect("listing is in project directory");

    // cleanup
    remove_file(bin_path).unwrap();
    remove_file(dasm_path).unwrap();

    assert_eq!(new, src);
}

// #[test]
// fn file_validation() {
//     validate_listing("listing0")
// }

/// Takes a listing name (eg. "listing37")
/// - assembles its asm file
/// - dissassembles the binary and saves it to a file
/// - assembles the dissassembly and finaly compares the two binaries
fn validate_listing(listing: &str) {
    // assemble the listing
    let src_path = format!("{listing}.asm");
    Command::new("nasm").args([&src_path]).output().unwrap();

    // disassemble
    let src = fs::read(listing).expect("listing is in project directory");
    remove_file(listing).unwrap();
    let dasm = disassemble(&src);

    let dasm_path = format!("{listing}-d.asm");
    let _ = fs::write(&dasm_path, dasm);

    // assemble the disassembly
    Command::new("nasm").args([&dasm_path]).output().unwrap();

    let bin_path = format!("{listing}-d");
    let new = fs::read(&bin_path).expect("listing is in project directory");

    // cleanup
    remove_file(bin_path).unwrap();
    remove_file(dasm_path).unwrap();

    assert_eq!(new, src);
}
