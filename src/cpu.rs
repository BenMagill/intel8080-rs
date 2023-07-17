use crate::utils::{merge_bytes, self};

const MEMORY_SIZE: usize = 65_535;

pub struct I8080 {
    pub flags: StatusFlags,
    pub registers: Registers,
    pub memory: [u8; MEMORY_SIZE],
}

#[derive(Debug)]
pub struct StatusFlags {
    pub Z: bool,
    pub C: bool,
    pub S: bool,
    pub P: bool,
    pub AC: bool,
}

#[derive(Debug)]
pub struct Registers {
    pub A: u8,
    pub B: u8,
    pub C: u8,
    pub D: u8,
    pub E: u8,
    pub H: u8,
    pub L: u8,

    pub PC: u16,

    // Decrememnts when data pushed on,
    // Increments when popped
    pub SP: u16,
}

impl I8080 {
    pub fn new() -> I8080 {
        I8080 {
            registers: Registers {
                A: 0,
                B: 0,
                C: 0,
                D: 0,
                E: 0,
                H: 0,
                L: 0,
                PC: 0,
                SP: 0,
            },
            flags: StatusFlags {
                Z: false,
                C: false,
                S: false,
                P: false,
                AC: false,
            },
            memory: [0; MEMORY_SIZE]
        }
    }
    
    pub fn load(&mut self, buffer: &Vec<u8>) {
        for (pos, byte) in buffer.iter().enumerate() {
            self.memory[pos] = *byte;
        }
    }

    fn execute_cycle(&mut self) {
        let opcode = self.get_next_byte();

        match opcode {
            0x00 => {},
            0x76 => unimplemented!("halt"),

            // Data transfer
            0x40 ..= 0x7f => { // MOV
                // MOV r1, r2 (copy r2 to r1)
                let dest = (opcode >> 3) & 0b111;
                let src = opcode & 0b111;
                let r2 = self.get_source(src);
                self.set_dest(dest, r2);
            }, 
            0x06 | 0x0e | 0x16 | 0x1e | 0x26 | 0x2e | 0x36 | 0x3e => { // MVI
                let data = self.get_next_byte();
                let dest = (opcode >> 3) & 0b111;

                self.set_dest(dest, data);
            }
            0x01 | 0x11 | 0x21 | 0x31 => { // LXI
                let low = self.get_next_byte();
                let high = self.get_next_byte();
                let rp = (opcode >> 4) & 0b11;
                match rp {
                    0b00 => { // BC
                        self.registers.B = high;
                        self.registers.C = low;
                    },
                    0b01 => { // DE
                        self.registers.D= high;
                        self.registers.E = low;
                    },
                    0b10 => { // HL
                        self.registers.H = high;
                        self.registers.L = low;
                    },
                    0b11 => { // SP
                        self.registers.SP = merge_bytes(high, low);
                    },
                    _ => unreachable!(),
                };
            }
            0x3a => { // LDA
                let low = self.get_next_byte();
                let high = self.get_next_byte();
                let addr = merge_bytes(high, low);
                self.registers.A = self.memory[addr as usize];
            }
            0x32 => { // STA
                let low = self.get_next_byte();
                let high = self.get_next_byte();
                let addr = merge_bytes(high, low);
                self.memory[addr as usize] = self.registers.A;
            }
            0x2a => { // LHLD
                let low = self.get_next_byte();
                let high = self.get_next_byte();
                let addr = merge_bytes(high, low);
                self.registers.L = self.memory[addr as usize];
                self.registers.H = self.memory[(addr+1) as usize];
            }
            0x22 => { // SHLD
                let low = self.get_next_byte();
                let high = self.get_next_byte();
                let addr = merge_bytes(high, low);
                self.memory[addr as usize] = self.registers.L;
                self.memory[(addr+1) as usize] = self.registers.H; 
            }
            0x0a | 0x1a => { // LDAX
                let rp = (opcode >> 4) & 0b11;
                let (high, low) = match rp {
                    0b00 => { // BC
                        (self.registers.B, self.registers.C)
                    },
                    0b01 => { // DE
                        (self.registers.D, self.registers.E)
                    },
                    _ => unreachable!(),
                };
                let addr = merge_bytes(high, low);
                self.registers.A = self.memory[addr as usize];
            }
            0x02 | 0x12 => { // STAX
                let rp = (opcode >> 4) & 0b11;
                let (high, low) = match rp {
                    0b00 => { // BC
                        (self.registers.B, self.registers.C)
                    },
                    0b01 => { // DE
                        (self.registers.D, self.registers.E)
                    },
                    _ => unreachable!(),
                };
                let addr = merge_bytes(high, low);
                self.memory[addr as usize] = self.registers.A;
            }
            0xeb => { // XCHG
                let h = self.registers.H;
                let l = self.registers.L;
                self.registers.H = self.registers.D;
                self.registers.L = self.registers.E;
                self.registers.D = h;
                self.registers.E = l;
            }

            // Arithmetic
            0x80..=0x87 => { // ADD
                let src = opcode & 0b111;
                let r2 = self.get_source(src);

                self.add(r2);
            }
            0xc6 => { // ADI
                let data = self.get_next_byte();
                self.add(data);
            }
            0x88..=0x8f => { // ADC
                let src = opcode & 0b111;
                let r2 = self.get_source(src);
                let carry_bit = match self.flags.C {
                    true => 1,
                    false => 0,
                };

                self.add(r2 + carry_bit);
            }
            0x0ce => { // ACI
                let data = self.get_next_byte();
                let carry_bit = match self.flags.C {
                    true => 1,
                    false => 0,
                };

                self.add(data + carry_bit);
            }
            0x90..=0x97 => { // SUB
                let src = opcode & 0b111;
                let data = self.get_source(src);
                
                self.sub(data);    
            }
            0x04 | 0x0c | 0x14 | 0x1c | 0x24 | 0x2c | 0x34 | 0x3c => { // INR
                // TODO: reuse flag checking from add? 
                let dest = (opcode >> 3) & 0b111;
                let value = self.get_source(dest);
                
                let result = value as u16 + 1;
                let result_u8 = result as u8;

                if result_u8 == 0 {
                    self.flags.Z = true;
                } else {
                    self.flags.Z = false;
                }

                if result_u8 & 0x80 == 1 {
                    self.flags.S = true;    
                } else {
                    self.flags.S = false;
                }

                if utils::check_even_parity(result_u8) {
                    self.flags.P = true;
                } else {
                    self.flags.P = false;
                } 

                if (self.registers.A & 0x0f) + (value & 0x0f) > 0x0f {
                    self.flags.AC = true;
                } else {
                    self.flags.AC = false;
                }

                self.set_dest(dest, result_u8);
            }
            0x03 | 0x13 | 0x23 | 0x33 => { // INX
                let source = (opcode >> 4) & 0b11;
                let (high, low) = self.get_source_pair(source);
                let merged = merge_bytes(high, low);
                let sum = merged+1;
                let split = sum.to_be_bytes();
                let low = split[1];
                let high = split[0];
                let rp = (opcode >> 4) & 0b11;
                match rp {
                    0b00 => { // BC
                        self.registers.B = high;
                        self.registers.C = low;
                    },
                    0b01 => { // DE
                        self.registers.D= high;
                        self.registers.E = low;
                    },
                    0b10 => { // HL
                        self.registers.H = high;
                        self.registers.L = low;
                    },
                    0b11 => { // SP
                        self.registers.SP = sum;
                    },
                    _ => unreachable!(),
                };
            }

            // Branching
            0xc3 => { // JMP
                self.jmp();
            },
            0xc2 => { // JNZ
                if !self.flags.Z {
                    self.jmp();
                }
            }
            0xca => { // JZ
                if self.flags.Z {
                    self.jmp();
                }
            }
            0xd2 => { // JNC
                if !self.flags.C {
                    self.jmp();
                }
            }
            0xda => { // JC
                if self.flags.C {
                    self.jmp();
                }
            }
            0xe2 => { // JPO
                if !self.flags.P {
                    self.jmp();
                }
            }
            0xea => { // JPE
                if self.flags.P {
                    self.jmp();
                }
            }
            0xf2 => { // JP
                if !self.flags.S {
                    self.jmp();
                }
            }
            0xfa => { // JM
                if self.flags.S {
                    self.jmp();
                }
            }
            0xcd => { // CALL
                self.call();
            }
            0xc4 => { // CNZ
                if !self.flags.Z {
                    self.call();
                }
            }
            0xcc => { // CZ
                if self.flags.Z {
                    self.call();
                }
            }
            0xd4 => { // CNC
                if !self.flags.C {
                    self.call();
                }
            }
            0xdc => { // CC
                if self.flags.C {
                    self.call();
                }
            }
            0xe4 => { // CPO
                if !self.flags.P {
                    self.call();
                }
            }
            0xec => { // CPE
                if self.flags.P {
                    self.call();
                }
            }
            0xf4 => { // CP
                if !self.flags.S {
                    self.call();
                }
            }
            0xfc => { // CM
                if self.flags.S {
                    self.call();
                }
            }
            0xc9 => { // RET
                self.ret();
            }
            0xc0 => { // RNZ
                if !self.flags.Z {
                    self.ret();
                }
            }
            0xc8 => { // RZ
                if self.flags.Z {
                    self.ret();
                }
            }
            0xd0 => { // RNC
                if !self.flags.C {
                    self.ret();
                }
            }
            0xd8 => { // RC
                if self.flags.C {
                    self.ret();
                }
            }
            0xe0 => { // RPO
                if !self.flags.P {
                    self.ret();
                }
            }
            0xe8 => { // RPE
                if self.flags.P {
                    self.ret();
                }
            }
            0xf0 => { // RP
                if !self.flags.S {
                    self.ret();
                }
            }
            0xf8 => { // RM
                if self.flags.S {
                    self.ret();
                }
            }
            0xc7 | 0xcf | 0xd7 | 0xdf | 0xe7 | 0xef | 0xf7 | 0xff => { // RST
                self.rst(opcode);
            }
            0xe9 => {
                self.registers.PC = merge_bytes(self.registers.H, self.registers.L);
            }


            _ => {self.debug_state(); unimplemented!("0x{:02x}", opcode)},
        };
    }

    pub fn run(&mut self) {
        loop {
            self.execute_cycle();
        }
    }

    // Reads the byte pointed to by the PC and increments it
    fn get_next_byte(&mut self) -> u8 {
        let pc = self.registers.PC;
        let byte = self.memory[pc as usize];
        self.registers.PC = pc + 1;
        byte
    }

    fn jmp(&mut self) {
        let b2 = self.get_next_byte();
        let b3 = self.get_next_byte();
        let addr = merge_bytes(b3, b2);
        self.registers.PC = addr;
    }
    
    fn call(&mut self) {
        let sp = self.registers.SP;
        let pc = self.registers.PC.to_be_bytes();
        self.memory[(sp-1) as usize] = pc[0];
        self.memory[(sp-2) as usize] = pc[1]; 
        self.registers.SP = sp - 2;
        self.jmp();            
    }

    fn ret(&mut self) {
        let sp = self.registers.SP;
        let sp_low = self.memory[sp as usize];
        let sp_high = self.memory[(sp+1) as usize];
        let pc = merge_bytes(sp_high, sp_low);
        self.registers.PC = pc;
        self.registers.SP = sp + 2;
    }

    fn rst(&mut self, opcode: u8) {
        let sp = self.registers.SP;
        let pc = self.registers.PC.to_be_bytes();
        self.memory[(sp-1) as usize] = pc[0];
        self.memory[(sp-2) as usize] = pc[1]; 
        self.registers.SP = sp - 2;
        let pc = (opcode & 0b111000) as u16;
        self.registers.PC = pc;
    }

    // Add a value to acc and update flags
    fn add(&mut self, value: u8) {
        // Prevent overflow
        let result = self.registers.A as u16 + value as u16;
        let result_u8 = result as u8;

        if result_u8 == 0 {
            self.flags.Z = true;
        } else {
            self.flags.Z = false;
        }

        if result_u8 & 0x80 == 1 {
            self.flags.S = true;    
        } else {
            self.flags.S = false;
        }

        if result > 0xff {
            self.flags.C = true;
        } else {
            self.flags.C = false;
        }

        if utils::check_even_parity(result_u8) {
            self.flags.P = true;
        } else {
            self.flags.P = false;
        } 

        if (self.registers.A & 0x0f) + (value & 0x0f) > 0x0f {
            self.flags.AC = true;
        } else {
            self.flags.AC = false;
        }

        self.registers.A = result_u8;
    }

    fn sub(&mut self, data: u8) {
        unimplemented!();
    }

    fn get_source_pair(&self, source: u8) -> (u8, u8) {
        match source {
            0b00 => { // BC
                (self.registers.B, self.registers.C)
            },
            0b01 => { // DE
                (self.registers.D, self.registers.E)
            },
            0b10 => { // HL
                (self.registers.H, self.registers.L)
            },
            0b11 => { // SP
                let sp = self.registers.SP.to_be_bytes();
                (sp[0], sp[1])
            },
            _ => unreachable!(),
        }
    }

    fn get_source(&self, source: u8) -> u8 {
        match source {
            0b000 => self.registers.B,
            0b001 => self.registers.C,
            0b010 => self.registers.D,
            0b011 => self.registers.E,
            0b100=> self.registers.H,
            0b101=> self.registers.L,
            0b110 => self.memory[merge_bytes(self.registers.H, self.registers.L) as usize],
            0b111 => self.registers.A,
            _ => unreachable!(),
        }
    }

    fn set_dest(&mut self, dest: u8, data: u8) {
        match dest {
            0b000 => self.registers.B = data,
            0b001 => self.registers.C = data,
            0b010 => self.registers.D = data,
            0b011 => self.registers.E = data,
            0b100 => self.registers.H = data,
            0b101 => self.registers.L = data,
            0b110 => self.memory[merge_bytes(self.registers.H, self.registers.L) as usize] = data,
            0b111 => self.registers.A = data,
            _ => unreachable!(),
        };
    }

    pub fn debug_state(&self) {
        dbg!(&self.registers);
        dbg!(&self.flags);
    }
}
