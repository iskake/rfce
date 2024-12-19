use crate::fc::dbg::Debugger;

pub mod bits;
pub mod fc;

fn main() {
    println!("Running the cpu tester!");
    let mut debugger = Debugger::new();
    debugger.run();
}
