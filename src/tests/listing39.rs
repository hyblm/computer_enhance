use super::validate_asm;

#[test]
fn byte_immediate_to_register() {
    validate_asm(
        "
        bits 16

        mov cl, 12
        mov ch, -12
        ",
    );
}

#[test]
fn word_immediate_to_register() {
    validate_asm(
        "
        bits 16

        mov cx, 12
        mov cx, -12
        ",
    );
}
#[test]
fn source_address_calculation() {
    validate_asm(
        "
        bits 16

        mov al, [bx + si]
        mov bx, [bp + di]
        mov dx, [bp]
        ",
    );
}
#[test]
fn source_address_calculation_byte_offset() {
    validate_asm(
        "
        bits 16

        mov al, [bx + si + 4]
        ",
    );
}
#[test]
fn source_address_calculation_word_offset() {
    validate_asm(
        "
        bits 16

        mov al, [bx + si + 4999]
        ",
    );
}
