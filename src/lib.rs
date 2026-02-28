const MEMORY_MAX: usize = 1 << 16;
const PC_START: u16 = 0x3000;

const MR_KBSR: u16 = 0xFE00;
const MR_KBDR: u16 = 0xFE02;

const FL_POS: u16 = 1 << 0;
const FL_ZRO: u16 = 1 << 1;
const FL_NEG: u16 = 1 << 2;

pub const R_R0: usize = 0;
pub const R_R1: usize = 1;
pub const R_R2: usize = 2;
pub const R_R3: usize = 3;
pub const R_R4: usize = 4;
pub const R_R5: usize = 5;
pub const R_R6: usize = 6;
pub const R_R7: usize = 7;
const R_PC: usize = 8;
const R_COND: usize = 9;
const R_COUNT: usize = 10;

const OP_BR: u16 = 0;
const OP_ADD: u16 = 1;
const OP_LD: u16 = 2;
const OP_ST: u16 = 3;
const OP_JSR: u16 = 4;
const OP_AND: u16 = 5;
const OP_LDR: u16 = 6;
const OP_STR: u16 = 7;
const OP_RTI: u16 = 8;
const OP_NOT: u16 = 9;
const OP_LDI: u16 = 10;
const OP_STI: u16 = 11;
const OP_JMP: u16 = 12;
const OP_RES: u16 = 13;
const OP_LEA: u16 = 14;
const OP_TRAP: u16 = 15;

const TRAP_GETC: u16 = 0x20;
const TRAP_OUT: u16 = 0x21;
const TRAP_PUTS: u16 = 0x22;
const TRAP_IN: u16 = 0x23;
const TRAP_PUTSP: u16 = 0x24;
const TRAP_HALT: u16 = 0x25;
