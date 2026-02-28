use lc3_vm::{R_R0, R_R1, R_R2, R_R3, R_R4, Vm};

fn image(origin: u16, words: &[u16]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity((words.len() + 1) * 2);
    bytes.extend_from_slice(&origin.to_be_bytes());
    for word in words {
        bytes.extend_from_slice(&word.to_be_bytes());
    }
    bytes
}

fn mask(value: i16, bits: u16) -> u16 {
    let mask = (1i32 << bits) - 1;
    (value as i32 & mask) as u16
}

fn add_imm(dr: u16, sr1: u16, imm5: i16) -> u16 {
    0x1000 | (dr << 9) | (sr1 << 6) | (1 << 5) | mask(imm5, 5)
}

fn and_imm(dr: u16, sr1: u16, imm5: i16) -> u16 {
    0x5000 | (dr << 9) | (sr1 << 6) | (1 << 5) | mask(imm5, 5)
}

fn br(n: bool, z: bool, p: bool, pc_offset9: i16) -> u16 {
    let cond = (u16::from(n) << 2) | (u16::from(z) << 1) | u16::from(p);
    (cond << 9) | mask(pc_offset9, 9)
}

fn jsr(pc_offset11: i16) -> u16 {
    0x4800 | mask(pc_offset11, 11)
}

fn jmp(base: u16) -> u16 {
    0xC000 | (base << 6)
}

fn not(dr: u16, sr: u16) -> u16 {
    0x9000 | (dr << 9) | (sr << 6) | 0x3F
}

fn ld(dr: u16, pc_offset9: i16) -> u16 {
    0x2000 | (dr << 9) | mask(pc_offset9, 9)
}

fn st(sr: u16, pc_offset9: i16) -> u16 {
    0x3000 | (sr << 9) | mask(pc_offset9, 9)
}

fn ldi(dr: u16, pc_offset9: i16) -> u16 {
    0xA000 | (dr << 9) | mask(pc_offset9, 9)
}

fn sti(sr: u16, pc_offset9: i16) -> u16 {
    0xB000 | (sr << 9) | mask(pc_offset9, 9)
}

fn ldr(dr: u16, base: u16, offset6: i16) -> u16 {
    0x6000 | (dr << 9) | (base << 6) | mask(offset6, 6)
}

fn str(sr: u16, base: u16, offset6: i16) -> u16 {
    0x7000 | (sr << 9) | (base << 6) | mask(offset6, 6)
}

fn lea(dr: u16, pc_offset9: i16) -> u16 {
    0xE000 | (dr << 9) | mask(pc_offset9, 9)
}

fn trap(vector: u16) -> u16 {
    0xF000 | vector
}

#[test]
fn vm_core_integration_program() {
    let words = [
        and_imm(0, 0, 0),
        add_imm(0, 0, 5),
        add_imm(1, 0, -5),
        br(false, true, false, 1),
        add_imm(2, 2, 1),
        jsr(2),
        add_imm(3, 3, 1),
        br(true, true, true, 3),
        not(4, 0),
        add_imm(4, 4, 1),
        jmp(7),
        lea(0, 10),
        ld(1, 6),
        str(1, 0, 0),
        ldr(2, 0, 0),
        st(2, 7),
        ldi(3, 3),
        sti(3, 3),
        trap(0x25),
        0x1234,
        0x3017,
        0x3018,
        0x0000,
        0x0000,
        0x0000,
    ];

    let mut vm = Vm::new();
    vm.load_image_bytes(&image(0x3000, &words)).unwrap();
    vm.run().unwrap();

    assert_eq!(vm.register(R_R0), 0x3016);
    assert_eq!(vm.register(R_R1), 0x1234);
    assert_eq!(vm.register(R_R2), 0x1234);
    assert_eq!(vm.register(R_R3), 0x1234);
    assert_eq!(vm.register(R_R4), 0xFFFB);
    assert_eq!(vm.memory_word(0x3016), 0x1234);
    assert_eq!(vm.memory_word(0x3017), 0x1234);
    assert_eq!(vm.memory_word(0x3018), 0x1234);
}
