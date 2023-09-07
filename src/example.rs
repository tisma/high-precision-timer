use std::thread;
use std::time::{self};
use std::env::consts;

fn main() {
    println!("Example of measurement of tick counter");
    let duration = time::Duration::from_secs(1);
    let start = high_precision_timer::start();
    thread::sleep(duration);
    let elapsed_ticks = high_precision_timer::stop() - start;
    println!("Elapsed ticks in {:?}: {}", duration, elapsed_ticks);
}