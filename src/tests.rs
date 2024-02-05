use crate::disassemble;
use rand::{distributions::Alphanumeric, Rng};
use std::{
    fs::{self, remove_file},
    process::Command,
};

#[test]
fn file_validation() {
    validate_listing("listing0")
}

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

// #[test]
// fn listing40() {
//     validate_listing("listing40")
// }

mod listing39;
// mod listing40;

/// Takes a listing name (eg. "listing37")
/// - assembles its asm file
/// - dissassembles the binary and saves it to a file
/// - assembles the dissassembly and finaly compares the two binaries
fn validate_listing(listing: &str) {
    let src_path = format!("./listings/{}.asm", listing);

    let (bin1, bin2) = produce_bins(&src_path);
    assert_eq!(bin1, bin2);
}
fn validate_asm(asm: &str) {
    println!("{}", asm);

    let binary_path: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    let src_path = &format!("listings/tmp/{}.asm", binary_path);

    fs::write(src_path, asm).unwrap();
    let (dasm_bin, asm_bin) = produce_bins(src_path);

    if let Err(err) = remove_file(src_path) {
        eprintln!("{}", err);
    };

    assert_eq!(dasm_bin, asm_bin);
}
fn produce_bins(src_path: &str) -> (Vec<u8>, Vec<u8>) {
    let produce_bin = |src_path, bin_path| {
        Command::new("nasm")
            .args([src_path])
            .output()
            .and_then(|_| fs::read(bin_path))
    };
    let (src_path, bin_path) = src_bin_paths(src_path);
    let src_path2 = format!("{}-d.asm", bin_path);
    let (src_path2, bin_path2) = src_bin_paths(&src_path2);

    let bin1 = produce_bin(src_path, bin_path).expect("Assembly compiled and read");

    let bin2 =
        fs::write(src_path2, disassemble(&bin1)).and_then(|_| produce_bin(src_path2, bin_path2));

    if let Err(err) = remove_file(src_path2) {
        eprintln!("{}", err);
    };
    if let Err(err) = remove_file(bin_path) {
        eprintln!("{}", err);
    };
    if let Err(err) = remove_file(bin_path2) {
        eprintln!("{}", err);
    };

    (bin1, bin2.expect("Disassembly compiled and read"))
}

fn src_bin_paths(src_path: &str) -> (&str, &str) {
    (src_path, src_path.get(..src_path.len() - 4).unwrap())
}
