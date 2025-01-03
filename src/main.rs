use std::env;

// use gui::Gui;

use crate::fc::dbg::Debugger;

pub mod bits;
pub mod fc;
pub mod gui;

fn main() -> Result<(), String> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    if let Some(filename) = args.get(1) {
        let mut debugger = Debugger::new();
        match debugger.load_file(filename) {
            Ok(_) => Ok(debugger.run()),
            Err(e) => Err(format!("{} (filename: '{}')", e, filename)),
        }
    } else {
        println!("No nes file provided.\n");
        println!("Usage: rfce <file>");
        Ok(())
    }

    // Temp., todo, etc.
    // let mut gui = Gui::new();
    // gui.run_forever();
}
