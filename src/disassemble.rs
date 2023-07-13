use std::{env, fs::File, process::exit, vec};
use std::io::prelude::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];

    println!("{}", file_path);

    let mut file = match File::open(file_path) {
       Ok(file) => file,
       Err(_) => {
           terminate("Could not open file");
       }
    };

    let mut buffer: Vec<u8> = Vec::new();

    file.read_to_end(&mut buffer).unwrap();

    println!("{:x?}", buffer);

    disassembler(&buffer);
}

fn disassembler(buffer: &Vec<u8>) {
    let mut offset = 0;
    while offset <= buffer.len() {
        let seek = disassemble_instr(buffer, offset);
        offset = (offset + 1) + seek as usize;
    }
}

// Returns size of operand (how much extra to seek by)
fn disassemble_instr(buffer: &Vec<u8>, offset: usize) -> u8 {
    let opcode = *match buffer.get(offset) {
        Some(opcode) => opcode,
        None => {
            terminate("Overflow");
        }
    };
    
    let mut seek = 0;

    match opcode {
       0x00 => println!("NOP"), 
       0x01 => {println!("LXI"); seek = 2},

       0xc3 => {println!("JMP {:x} {:x} {:x}", buffer[offset+1], buffer[offset+2], buffer[offset+3]); seek = 3},
        _ => terminate("Unexpected instruction"),
    }

    seek
}

fn terminate(message: &str) -> ! {
    eprintln!("{}", message);
    exit(1);
}
