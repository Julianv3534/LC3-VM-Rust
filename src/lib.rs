use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::io::{self, Read, Write};
use std::mem;
use std::path::Path;

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
pub struct Vm {
    memory: [u16; MEMORY_MAX],
    reg: [u16; R_COUNT],
    running: bool,
}

impl Default for Vm {
    fn default() -> Self {
        let mut reg = [0; R_COUNT];
        reg[R_PC] = PC_START;
        reg[R_COND] = FL_ZRO;

        Self {
            memory: [0; MEMORY_MAX],
            reg,
            running: true,
        }
    }
}

impl Vm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, index: usize) -> u16 {
        self.reg[index]
    }

    pub fn memory_word(&self, address: u16) -> u16 {
        self.memory[address as usize]
    }

    pub fn load_image_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), VmError> {
        let bytes = fs::read(path)?;
        self.load_image_bytes(&bytes)
    }

    pub fn load_image_bytes(&mut self, bytes: &[u8]) -> Result<(), VmError> {
        if bytes.len() < 2 {
            return Err(VmError::ImageTooSmall);
        }

        let origin = u16::from_be_bytes([bytes[0], bytes[1]]);
        let words = (bytes.len() - 2) / 2;
        let start = origin as usize;
        let end = start + words;
        if end > MEMORY_MAX {
            return Err(VmError::ImageTooLarge { origin, words });
        }

        for (i, chunk) in bytes[2..].chunks_exact(2).enumerate() {
            self.memory[start + i] = u16::from_be_bytes([chunk[0], chunk[1]]);
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), VmError> {
        self.running = true;
        while self.running {
            let instr = self.mem_read(self.reg[R_PC])?;
            self.reg[R_PC] = self.reg[R_PC].wrapping_add(1);

            match instr >> 12 {
                OP_ADD => self.op_add(instr),
                OP_AND => self.op_and(instr),
                OP_NOT => self.op_not(instr),
                OP_BR => self.op_br(instr),
                OP_JMP => self.op_jmp(instr),
                OP_JSR => self.op_jsr(instr),
                OP_LD => self.op_ld(instr)?,
                OP_LDI => self.op_ldi(instr)?,
                OP_LDR => self.op_ldr(instr)?,
                OP_LEA => self.op_lea(instr),
                OP_ST => self.op_st(instr),
                OP_STI => self.op_sti(instr)?,
                OP_STR => self.op_str(instr),
                OP_TRAP => self.op_trap(instr)?,
                OP_RTI | OP_RES => return Err(VmError::InvalidOpcode(instr >> 12)),
                _ => return Err(VmError::InvalidOpcode(instr >> 12)),
            }
        }

        Ok(())
    }

    fn op_add(&mut self, instr: u16) {
        let dr = ((instr >> 9) & 0x7) as usize;
        let sr1 = ((instr >> 6) & 0x7) as usize;

        if ((instr >> 5) & 1) != 0 {
            self.reg[dr] = self.reg[sr1].wrapping_add(sign_extend(instr & 0x1F, 5));
        } else {
            let sr2 = (instr & 0x7) as usize;
            self.reg[dr] = self.reg[sr1].wrapping_add(self.reg[sr2]);
        }
        self.update_flags(dr);
    }

    fn op_and(&mut self, instr: u16) {
        let dr = ((instr >> 9) & 0x7) as usize;
        let sr1 = ((instr >> 6) & 0x7) as usize;

        if ((instr >> 5) & 1) != 0 {
            self.reg[dr] = self.reg[sr1] & sign_extend(instr & 0x1F, 5);
        } else {
            let sr2 = (instr & 0x7) as usize;
            self.reg[dr] = self.reg[sr1] & self.reg[sr2];
        }
        self.update_flags(dr);
    }

    fn op_not(&mut self, instr: u16) {
        let dr = ((instr >> 9) & 0x7) as usize;
        let sr = ((instr >> 6) & 0x7) as usize;
        self.reg[dr] = !self.reg[sr];
        self.update_flags(dr);
    }

    fn op_br(&mut self, instr: u16) {
        let pc_offset = sign_extend(instr & 0x1FF, 9);
        let cond_flag = (instr >> 9) & 0x7;
        if (cond_flag & self.reg[R_COND]) != 0 {
            self.reg[R_PC] = self.reg[R_PC].wrapping_add(pc_offset);
        }
    }

    fn op_jmp(&mut self, instr: u16) {
        let base = ((instr >> 6) & 0x7) as usize;
        self.reg[R_PC] = self.reg[base];
    }

    fn op_jsr(&mut self, instr: u16) {
        self.reg[R_R7] = self.reg[R_PC];
        if ((instr >> 11) & 1) != 0 {
            self.reg[R_PC] = self.reg[R_PC].wrapping_add(sign_extend(instr & 0x7FF, 11));
        } else {
            let base = ((instr >> 6) & 0x7) as usize;
            self.reg[R_PC] = self.reg[base];
        }
    }

    fn op_ld(&mut self, instr: u16) -> Result<(), VmError> {
        let dr = ((instr >> 9) & 0x7) as usize;
        let address = self.reg[R_PC].wrapping_add(sign_extend(instr & 0x1FF, 9));
        self.reg[dr] = self.mem_read(address)?;
        self.update_flags(dr);
        Ok(())
    }

    fn op_ldi(&mut self, instr: u16) -> Result<(), VmError> {
        let dr = ((instr >> 9) & 0x7) as usize;
        let address = self.reg[R_PC].wrapping_add(sign_extend(instr & 0x1FF, 9));
        let indirect = self.mem_read(address)?;
        self.reg[dr] = self.mem_read(indirect)?;
        self.update_flags(dr);
        Ok(())
    }

    fn op_ldr(&mut self, instr: u16) -> Result<(), VmError> {
        let dr = ((instr >> 9) & 0x7) as usize;
        let base = ((instr >> 6) & 0x7) as usize;
        let address = self.reg[base].wrapping_add(sign_extend(instr & 0x3F, 6));
        self.reg[dr] = self.mem_read(address)?;
        self.update_flags(dr);
        Ok(())
    }

    fn op_lea(&mut self, instr: u16) {
        let dr = ((instr >> 9) & 0x7) as usize;
        self.reg[dr] = self.reg[R_PC].wrapping_add(sign_extend(instr & 0x1FF, 9));
        self.update_flags(dr);
    }

    fn op_st(&mut self, instr: u16) {
        let sr = ((instr >> 9) & 0x7) as usize;
        let address = self.reg[R_PC].wrapping_add(sign_extend(instr & 0x1FF, 9));
        self.mem_write(address, self.reg[sr]);
    }

    fn op_sti(&mut self, instr: u16) -> Result<(), VmError> {
        let sr = ((instr >> 9) & 0x7) as usize;
        let address = self.reg[R_PC].wrapping_add(sign_extend(instr & 0x1FF, 9));
        let indirect = self.mem_read(address)?;
        self.mem_write(indirect, self.reg[sr]);
        Ok(())
    }

    fn op_str(&mut self, instr: u16) {
        let sr = ((instr >> 9) & 0x7) as usize;
        let base = ((instr >> 6) & 0x7) as usize;
        let address = self.reg[base].wrapping_add(sign_extend(instr & 0x3F, 6));
        self.mem_write(address, self.reg[sr]);
    }

    fn op_trap(&mut self, instr: u16) -> Result<(), VmError> {
        self.reg[R_R7] = self.reg[R_PC];

        match instr & 0xFF {
            TRAP_GETC => {
                self.reg[R_R0] = read_char()?;
                self.update_flags(R_R0);
            }
            TRAP_OUT => write_char((self.reg[R_R0] & 0xFF) as u8)?,
            TRAP_PUTS => {
                let mut address = self.reg[R_R0];
                while self.memory[address as usize] != 0 {
                    write_char((self.memory[address as usize] & 0xFF) as u8)?;
                    address = address.wrapping_add(1);
                }
            }
            TRAP_IN => {
                write_str("Enter a character: ")?;
                let c = read_char()?;
                write_char((c & 0xFF) as u8)?;
                self.reg[R_R0] = c;
                self.update_flags(R_R0);
            }
            TRAP_PUTSP => {
                let mut address = self.reg[R_R0];
                while self.memory[address as usize] != 0 {
                    let val = self.memory[address as usize];
                    write_char((val & 0xFF) as u8)?;
                    let upper = (val >> 8) as u8;
                    if upper != 0 {
                        write_char(upper)?;
                    }
                    address = address.wrapping_add(1);
                }
            }
            TRAP_HALT => {
                write_str("HALT\n")?;
                self.running = false;
            }
            _ => {}
        }

        Ok(())
    }
