use std::{env, fs::File, process::exit};
use std::io::prelude::*;

pub fn load_file(buffer: &mut Vec<u8>) {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];

    let mut file = match File::open(file_path) {
       Ok(file) => file,
       Err(_) => {
           terminate("Could not open file");
       }
    };

    file.read_to_end(buffer).unwrap();
}

pub fn merge_bytes(left: u8, right: u8) -> u16 {
    (left as u16) << 8 | right as u16
}

pub fn terminate(message: &str) -> ! {
    println!("{}", message);
    exit(1);
}

pub fn check_even_parity(data: u8) -> bool {
    data.count_ones() % 2 == 0 
}
