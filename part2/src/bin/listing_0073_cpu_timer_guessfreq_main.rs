use std::env::args;

use part2::profile::estimate_cpu_frequency;

fn main() {
    let millis_to_wait = args().nth(1).and_then(|x| x.parse().ok()).unwrap_or(1000);
    let cpu_frequency = estimate_cpu_frequency(millis_to_wait);
    println!("Estimated CPU frequency: {cpu_frequency}");
}
