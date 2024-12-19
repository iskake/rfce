use std::io::{self, Write};
use std::time::Instant;

use crate::fc::CPU_FREQ;

use super::FC;

pub struct Debugger {
    fc: FC,
}

impl Debugger {
    pub fn new() -> Debugger {
        let fc = FC::new();
        Debugger { fc }
    }

    pub fn load_file(&mut self, filename: &str) -> () {
        self.fc.load_rom(filename);
    }

    pub fn run(&mut self) -> () {
        let mut input = String::new();

        println!("Started debugger");
        self.fc.cpu.print_state();
        loop {
            print!("> ");
            io::stdout().flush().unwrap();
            input.clear();

            let stdin = io::stdin();
            let mut start = Instant::now();
            match stdin.read_line(&mut input) {
                Ok(_) => {
                    let parts: Vec<&str> = input.trim().split(" ").collect();
                    // println!("{}", parts[0]);
                    let mut i: u64 = 0;
                    match parts[0] {
                        "c" => loop {
                            self.fc.cpu.fetch_and_run();
                            i += 1;
                            if i % (CPU_FREQ * 10) == 0 {
                                println!("{}, took: {}", i, start.elapsed().as_secs_f32());
                                start = Instant::now();
                            }
                        },
                        "s" => {
                            self.fc.cpu.fetch_and_run_dbg();
                            self.fc.cpu.print_state();
                        }
                        _ => println!("Unknown command: {}", parts[0]),
                    }
                }
                Err(err) => println!("An error occurred: {}", err),
            }
        }
    }
}
