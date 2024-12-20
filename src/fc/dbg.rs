use std::io::{self, Error, Write};
use std::time::Instant;

use crate::bits;
use crate::fc::CPU_FREQ;

use super::FC;

#[derive(PartialEq)]
enum Breakpoint {
    Address(u16),
}

impl std::fmt::Display for Breakpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Breakpoint::Address(addr) => write!(f, "at address ${addr:04x}"),
        }
    }
}

pub struct Debugger {
    fc: FC,
    last_input: String,
    breakpoints: Vec<Breakpoint>,
}

impl Debugger {
    pub fn new() -> Debugger {
        let fc = FC::new();
        Debugger {
            fc,
            last_input: String::from(""),
            breakpoints: Vec::new(),
        }
    }

    pub fn load_file(&mut self, filename: &str) -> Result<(), Error> {
        self.fc.load_rom(filename)?;
        self.fc.init();
        Ok(())
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
                    if input == "\n" {
                        input = self.last_input.clone();
                    } else {
                        self.last_input = input.clone();
                    }

                    let parts: Vec<&str> = input.trim().split(" ").collect();
                    // println!("{}", parts[0]);
                    let mut i: u64 = 0;
                    match parts[0] {
                        "c" => loop {   // Continue running
                            self.fc.cpu.fetch_and_run();
                            i += 1;
                            if i % (CPU_FREQ * 10) == 0 {
                                println!("{}, took: {}", i, start.elapsed().as_secs_f32());
                                start = Instant::now();
                            }

                            let addr_break = Breakpoint::Address(self.fc.cpu.pc());
                            if self.breakpoints.contains(&addr_break) {
                                println!("Hit breakpoint {}", addr_break);
                                self.fc.cpu.print_state();
                                break;
                            }
                        },
                        "s" => {    // Step cpu forward 1 instruction
                            self.fc.cpu.fetch_and_run_dbg();
                            self.fc.cpu.print_state();
                        },
                        "b" => {    // Add breakpoint
                            if parts.len() <= 1 {
                                println!("No address provided!");
                                println!("Usage: b address");
                            } else if !parts[1].starts_with("$") && !parts[1].starts_with("0x") {
                                println!("Address be prefixed with `$` or `0x`!");
                                println!("Usage: b address");
                            } else if let Ok(addr) = bits::parse_hex(parts[1]) {
                                self.breakpoints.push(Breakpoint::Address(addr));
                            } else {
                                println!("Could not parse provided address!");
                                println!("Usage: b address");
                            }
                        },
                        "d" => {    // Delete breakpoint
                            if parts.len() <= 1 {
                                println!("No address provided!");
                                println!("Usage: d address");
                            } else if !parts[1].starts_with("$") && !parts[1].starts_with("0x") {
                                println!("Address must start with `$` or `0x`!");
                                println!("Usage: d address");
                            } else if let Ok(addr) = bits::parse_hex(parts[1]) {
                                let breakpoint = Breakpoint::Address(addr);

                                if let Some(index) = self.breakpoints.iter().position(|x| *x == breakpoint) {
                                    self.breakpoints.remove(index);
                                } else {
                                    println!("Breakpoint does not exist: ${addr:04x}");
                                }
                            } else {
                                println!("Could not parse provided address!");
                                println!("Usage: d address");
                            }
                        },
                        "list" => {
                            self.breakpoints.iter().for_each(|x| println!("{x}"));
                        }
                        _ => println!("Unknown command: {}", parts[0]),
                    }
                }
                Err(err) => println!("An error occurred: {}", err),
            }
        }
    }
}
