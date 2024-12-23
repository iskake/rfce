use std::collections::HashSet;
use std::io::{self, Error, Write};
use std::process::exit;

use crate::bits;

use super::FC;

#[derive(PartialEq, Eq, Hash)]
enum Breakpoint {
    Address(u16),
    CPUCycle(u64),  // TODO
    PPUCycle(u64),  // TODO
    Scanline(u16),  // TODO
}

impl std::fmt::Display for Breakpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Breakpoint::Address(addr) => write!(f, "at address ${addr:04x}"),
            Breakpoint::CPUCycle(cycle) => write!(f, "at CPU cycle {cycle}"),
            Breakpoint::PPUCycle(cycle) => write!(f, "at PPU cycle {cycle}"),
            Breakpoint::Scanline(scanline) => write!(f, "at scanline {scanline} hit"),
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
        Ok(())
    }

    pub fn run(&mut self) -> () {
        let mut input = String::new();

        println!("Started debugger");
        self.fc.cpu.print_state();
        loop {
            print!("{}", if !self.ed_mode { "(dbg) " } else { "" });
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
        if *input == "\n" || *input == "\u{1b}[A" {
            *input = self.last_input.clone();
        } else {
            self.last_input = input.clone();
        }

        let parts: Vec<&str> = input.trim().split(" ").collect();

        match parts[..] {
            ["c"] => {
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
            ["s"] => {
                // Step cpu forward 1 instruction
                self.fc.cpu.fetch_and_run_dbg();
                self.fc.cpu.print_state();
                Ok(())
            }
            ["b" | "break", addr] => {
                self.handle_breakpoint_add(&parts, "address", addr)
            }
            ["b" | "break", break_type, val] => {
                self.handle_breakpoint_add(&parts, break_type, val)
            }
            ["b" | "break", ..] => Err(String::from("Usage: break [address|scanline] <value>")),
            ["d" | "delete", break_type, val] => {
                self.handle_breakpoint_del(break_type, val)
            }
            ["d" | "delete", ..] => Err(String::from("Usage: delete [address|scanline] <value>")),
            ["l" | "list"] => {
                // List breakpoints
                self.breakpoints.iter().for_each(|x| println!("{x}"));
                Ok(())
            }
            ["p" | "print"] => {
                // Print cpu info
                self.fc.cpu.print_state();
                Ok(())
            }
            ["r" | "hr" | "reload" | "run"] | ["hard", "reset"] => {
                // ? Would results in error if we allow debugger when no rom loaded
                let _ = self.fc.reset_hard();
                Ok(())
            }
            ["sr" | "reset"] | ["soft", "reset"] => {
                // Soft reset
                self.fc.reset();
                Ok(())
            }
            ["load", filename] => {
                if parts.len() <= 1 {
                    return Err(String::from(
                        "No file provided!\n\
                         Usage: load <file.nes>",
                    ));
                }
                match self.fc.load_rom(filename).into() {
                    Ok(_a) => Ok(_a),
                    Err(e) => Err(String::from(format!("Could not load the file: {e}"))),
                }
            }
            ["load", ..] => Err(String::from("Usage: load <filen.nes>")),
            ["x", mem_type, addr] => self.examine(mem_type, addr),
            ["x", addr] => self.examine("cpu", addr),
            ["x", ..] => Err(String::from("Usage: x $<address>")),
            ["q" | "quit" | "exit"] => {
                if !self.ed_mode {
                    exit(0)
                } else {
                    Err(String::new())
                }
            }
            ["?"] => {
                // Surely the most useful feature of this debugger
                self.ed_mode = true;
                Err("?".to_owned())
            }
            ["the", "standard"] => {
                if self.ed_mode {
                    // No actual reason to do this
                    self.ed_mode = false;
                    Ok(())
                } else {
                    Err(String::from(format!("Unknown command: `{:?}`", parts)))
                }
            }
            _ => Err(String::from(format!("Unknown command: `{:?}`", parts))),
        }
    }

    fn examine(&self, mem_type: &str, addr: &str) -> Result<(), String> {
        match mem_type {
            "cpu" => {
                let from_addr = (Self::try_parse_hex(addr)? & 0xfff0) as u32;
                let f = |a| self.fc.cpu.read_addr_nocycle(a);
                self.print_mem_region(from_addr, from_addr + 0x30, f);
                Ok(())
            },
            "ppu" => {
                let from_addr = (Self::try_parse_hex(addr)? & 0xfff0) as u32;
                let f = |a| self.fc.cpu.read_addr_ppu(a);
                self.print_mem_region(from_addr, from_addr + 0x30, f);
                Ok(())
            },
            _ => Err(String::from(format!("Invalid memory type: `{mem_type}`")))
        }
    }

    fn try_parse_hex(val: &str) -> Result<u16, String> {
        if !val.starts_with("$") && !val.starts_with("0x") {
            Err(String::from("Address must be prefixed with `$` or `0x`"))
        } else if let Ok(addr) = bits::parse_hex(val) {
            Ok(addr)
        } else {
            Err(String::from("Could not parse the provided address as a 16 bit hex number."))
        }
    }

    fn print_mem_region<T: Fn(u16) -> u8>(&self, from: u32, to: u32, f: T) -> () {
        for i in from..to {
            if i != from as u32 {
                if i % 0x10 == 0 {
                    print!("\n");
                } else {
                    print!(" ");
                }
            }

            if i % 0x10 == 0 {
                let addr = i as u16;
                print!("${addr:04x}:  ");
            }

            let m = f(i as u16);
            print!("{m:02x}");
        }
        println!();
    }

    fn handle_breakpoint_add(&mut self, parts: &Vec<&str>, break_type: &str, val: &str) -> Result<(), String> {
        // Add breakpoint
        match break_type {
            "a" | "addr" | "address" => {
                if !val.starts_with("$") && !val.starts_with("0x") {
                    Err(String::from(
                        "Address must be prefixed with `$` or `0x`!\n\
                         Usage: break address $<address>",
                    ))
                } else if let Ok(addr) = bits::parse_hex(val) {
                    self.try_add_breakpoint(Breakpoint::Address(addr))
                } else {
                    Err(String::from(
                        "Could not parse provided address!\n\
                         Usage: break address $<address>",
                    ))
                }
            },
            "s" | "scan" | "line" | "scanline" => {
                println!("INFO: breakpoints for scanlines currently do not work");
                // Add a breakpoint for a scanline
                if let Ok(scanline) = val.parse() {
                    self.try_add_breakpoint(Breakpoint::Scanline(scanline))
                } else {
                    Err(String::from(
                        "Failed to parse scanline number!\n\
                         Usage: break scanline <scanline number>",
                    ))
                }
            },
            _ => Err(String::from(
                format!("Invalid breakpoint type & number combo: `{break_type} {val}`\n\
                         Usage: break [address|scanline] <value>")
            ))
        }
    }

    fn handle_breakpoint_del(&mut self, break_type: &str, val: &str) -> Result<(), String> {
        // Add breakpoint
        match break_type {
            "a" | "addr" | "address" => {
                if !val.starts_with("$") && !val.starts_with("0x") {
                    Err(String::from(
                        "Address must be prefixed with `$` or `0x`!\n\
                         Usage: delete address $<address>",
                    ))
                } else if let Ok(addr) = bits::parse_hex(val) {
                    self.try_remove_breakpoint(Breakpoint::Address(addr))
                } else {
                    Err(String::from(
                        "Could not parse provided address!\n\
                         Usage: delete address $<address>",
                    ))
                }
            },
            "s" | "scan" | "line" | "scanline" => {
                println!("INFO: breakpoints for scanlines currently do not work");
                // Add a breakpoint for a scanline
                if let Ok(scanline) = val.parse() {
                    self.try_remove_breakpoint(Breakpoint::Scanline(scanline))
                } else {
                    Err(String::from(
                        "Failed to parse scanline number!\n\
                         Usage: delete scanline <scanline number>",
                    ))
                }
            },
            _ => Err(String::from(
                format!("Invalid number: {val}\n\
                         Usage: delete [address|scanline] <value>")
            ))
        }
    }

    fn try_add_breakpoint(&mut self, breakpoint: Breakpoint) -> Result<(), String> {
        if !self.breakpoints.contains(&breakpoint) {
            self.breakpoints.insert(breakpoint);
            Ok(())
        } else {
            Err(String::from(format!("Breakpoint already exists: {breakpoint}")))
        }
    }

    fn try_remove_breakpoint(&mut self, breakpoint: Breakpoint) -> Result<(), String> {
        if self.breakpoints.contains(&breakpoint) {
            self.breakpoints.remove(&breakpoint);
            Ok(())
        } else {
            Err(String::from(format!("Breakpoint does not exist: {breakpoint}")))
        }
    }
}
