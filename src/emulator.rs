use cpu::I8080;

mod cpu;
mod utils;
use utils::load_file;

fn main() {
    let mut buffer: Vec<u8> = Vec::new();

    load_file(&mut buffer);
    
    emulator(&buffer);
}

fn emulator(buffer: &Vec<u8>) {
    let mut i8080 = I8080::new();

    i8080.load(buffer);

    i8080.run();
}
