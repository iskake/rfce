use std::{env, io::Error};

use gui::Gui;

use crate::fc::dbg::Debugger;

pub mod bits;
pub mod fc;
pub mod gui;

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();

    if let Some(filename) = args.get(1) {
        let mut debugger = Debugger::new();
        debugger.load_file(filename)?;
        debugger.run();
    } else {
        println!("No nes file provided.\n");
        println!("Usage: rfce <file>\n");
    }

    // Temp., todo, etc.
    // let mut gui = Gui::new();
    // gui.run_forever();

    Ok(())
}
