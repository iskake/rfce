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
    let last_arg_is_nes_file = args[args.len() -1].ends_with(".nes");

    if headless {
        if last_arg_is_nes_file {
            info!("Starting headless debugger");

            let filename = &args[args.len() - 1];
            let mut debugger = Debugger::new();
            match debugger.load_file(Path::new(&filename)) {
                Ok(_) => Ok(debugger.run()),
                Err(e) => Err(format!(
                    "{} (file: '{}') Help: Did you specify a valid .nes file?",
                    e, filename
                )),
            }
        } else {
            println!("No nes file provided.\n");
            println!("Usage: rfce --headless <file>");
            Ok(())
        }
    } else {
        info!("Starting GUI");

        let mut gui = if last_arg_is_nes_file {
            let filename = &args[args.len() - 1];
            info!("Loading .nes file: {filename}");
            GUI::from_file(Path::new(filename))
        } else {
            info!("Starting without ROM");
            GUI::new()
        };
        gui.run_forever();
        Ok(())
    }
}
