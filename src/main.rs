use std::{env, path::Path};

use gui::GUI;

pub mod bits;
pub mod fc;
pub mod gui;

fn main() -> Result<(), String> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if let Some(filename) = args.get(1) {
        let gui = GUI::from_file(Path::new(filename));
        match gui {
            Ok(mut g) => Ok(g.run_forever()),
            Err(e) => Err(format!("{} (filename: '{}')", e, filename)),
        }
    } else {
        println!("No nes file provided.\n");
        println!("Usage: rfce <file>");
        Ok(())
    }
}
