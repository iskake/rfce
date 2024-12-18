use crate::fc::dbg::Debugger;

pub mod fc;
pub mod bits;

fn main() {
    println!("Running the cpu tester!");
    let mut debugger = Debugger::new();
    debugger.run();
}
