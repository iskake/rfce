use std::collections::HashSet;
use std::io::{self, Error, Write};
use std::process::exit;

use crate::bits;

use super::FC;

#[derive(PartialEq, Eq, Hash)]
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
    breakpoints: HashSet<Breakpoint>,
    ed_mode: bool, // :)
}

impl Debugger {
    pub fn new() -> Debugger {
        let fc = FC::new();
        Debugger {
            fc,
            last_input: String::from(""),
            breakpoints: HashSet::new(),
            ed_mode: false,
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
            print!("{}", if !self.ed_mode { "> " } else { "" });
            io::stdout().flush().unwrap();
            input.clear();

            let stdin = io::stdin();
            match stdin.read_line(&mut input) {
                Ok(_) => match self.handle_input(&mut input) {
                    Ok(()) => (),
                    Err(e) => println!("{}", if !self.ed_mode { e } else { "?".to_owned() }),
                },
                Err(err) => println!("An error occurred: {}", err),
            };
        }
    }

    fn handle_input(&mut self, input: &mut String) -> Result<(), String> {
        if *input == "\n" {
            *input = self.last_input.clone();
        } else {
            self.last_input = input.clone();
        }

        let parts: Vec<&str> = input.trim().split(" ").collect();

        match parts[0] {
            "c" => {
                loop {
                    // Continue running
                    self.fc.cpu.fetch_and_run();
                    let addr_break = Breakpoint::Address(self.fc.cpu.pc());
                    if self.breakpoints.contains(&addr_break) {
                        println!("Hit breakpoint {}", addr_break);
                        self.fc.cpu.print_state();
                        break;
                    }
                }
                Ok(())
            }
            "s" => {
                // Step cpu forward 1 instruction
                self.fc.cpu.fetch_and_run_dbg();
                self.fc.cpu.print_state();
                Ok(())
            }
            "b" => {
                // Add breakpoint
                if parts.len() <= 1 {
                    Err(String::from(
                        "No address provided!\n\
                         Usage: b address",
                    ))
                } else if !parts[1].starts_with("$") && !parts[1].starts_with("0x") {
                    Err(String::from(
                        "Address must be prefixed with `$` or `0x`!\n\
                         Usage: b address",
                    ))
                } else if let Ok(addr) = bits::parse_hex(parts[1]) {
                    let breakpoint = Breakpoint::Address(addr);

                    if !self.breakpoints.contains(&breakpoint) {
                        self.breakpoints.insert(breakpoint);
                        Ok(())
                    } else {
                        Err(String::from("Breakpoint already exists: ${addr:04x}"))
                    }
                } else {
                    Err(String::from(
                        "Could not parse provided address!\n\
                         Usage: b address",
                    ))
                }
            }
            "d" => {
                // Delete breakpoint
                if parts.len() <= 1 {
                    Err(String::from(
                        "No address provided!\n\
                         Usage: b address",
                    ))
                } else if !parts[1].starts_with("$") && !parts[1].starts_with("0x") {
                    Err(String::from(
                        "Address must be prefixed with `$` or `0x`!\n\
                         Usage: b address",
                    ))
                } else if let Ok(addr) = bits::parse_hex(parts[1]) {
                    let breakpoint = Breakpoint::Address(addr);

                    if self.breakpoints.contains(&breakpoint) {
                        self.breakpoints.remove(&breakpoint);
                        Ok(())
                    } else {
                        Err(String::from("Breakpoint does not exist: ${addr:04x}"))
                    }
                } else {
                    Err(String::from(
                        "Could not parse provided address!\n\
                         Usage: b address",
                    ))
                }
            }
            "list" => {
                // List breakpoints
                self.breakpoints.iter().for_each(|x| println!("{x}"));
                Ok(())
            }
            "q" | "quit" | "exit" => exit(0),
            "?" => {
                // Surely the most useful feature of this debugger
                self.ed_mode = true;
                Err("?".to_owned())
            }
            "the_standard" => {
                if self.ed_mode {
                    // No actual reason to do this
                    self.ed_mode = false;
                    Ok(())
                } else {
                    Err(String::from(format!("Unknown command: {}", parts[0])))
                }
            }
            _ => Err(String::from(format!("Unknown command: {}", parts[0]))),
        }
    }
}
