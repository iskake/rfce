use std::env;

use crate::fc::dbg::Debugger;

pub mod bits;
pub mod fc;

fn main() {
    let args: Vec<String> = env::args().collect();

    for (i, arg) in args.iter().enumerate() {
        println!("arg {i}: {arg}");
    }

    if let Some(filename) = args.get(1) {
        println!("Running the cpu tester!");
        let mut debugger = Debugger::new();
        debugger.load_file(filename);
        debugger.run();
    } else {
        println!("No nes file provided.\n");
        println!("Usage: rfce file.nes");
    }
}
