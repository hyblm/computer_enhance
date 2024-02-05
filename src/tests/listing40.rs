use super::validate_asm;

#[test]
fn signed_displacements() {
    validate_asm(
        "
bits 16

mov ax, [bx + di - 37]
mov [si - 300], cx
mov dx, [bx - 32]
        ",
    );
}

#[test]
fn explicit_sizes() {
    validate_asm(
        "
bits 16

mov [bp + di], byte 7
mov [di + 901], word 347
        ",
    );
}

#[test]
fn direct_address() {
    validate_asm(
        "
bits 16

mov bp, [5]
mov bx, [3458]
        ",
    );
}

#[test]
fn memory_to_accumulator() {
    validate_asm(
        "
bits 16

mov ax, [2555]
mov ax, [16]
        ",
    );
}

#[test]
fn accumulator_to_memeory() {
    validate_asm(
        "
bits 16

mov [2554], ax
mov [15], ax
        ",
    );
}
