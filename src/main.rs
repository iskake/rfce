use std::{env, path::Path};

use fc::dbg::Debugger;
use gui::GUI;
use log::info;

pub mod bits;
pub mod fc;
pub mod gui;

fn main() -> Result<(), String> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    let headless = args.contains(&"--headless".to_owned());

    if let Some(filename) = args.get(args.len() - 1) {
        if headless {
            info!("Starting headless debugger");

            let mut debugger = Debugger::new();
            match debugger.load_file(Path::new(filename)) {
                Ok(_) => Ok(debugger.run()),
                Err(e) => Err(format!("{} (filename: '{}')", e, filename)),
            }
        } else {
            info!("Starting GUI");

            let mut gui = GUI::from_file(Path::new(filename));
            gui.run_forever();
            Ok(())
        }
    } else {
        println!("No nes file provided.\n");
        println!("Usage: rfce [options] <file>");
        Ok(())
    }
}
