use std::{env::args, fs::File, io::Read, process::exit};

use haversine::{reference_haversine, EARTH_RADIUS};
use part2::json_parser::parse_haversine_pairs;
use part2::profile::{estimate_cpu_frequency, read_timer_cpu};
use part2::Pair;

fn main() {
    let timing_start = read_timer_cpu();
    let Some(path) = args().nth(1) else {
        eprintln!("No file received");
        exit(1);
    };

    let mut json_file = match File::open(path) {
        Ok(file) => file,
        Err(error) => {
            eprintln!("Failed to open file with error: {error}");
            exit(1);
        }
    };
    let timing_startup = read_timer_cpu();

    let json_length = match json_file.metadata() {
        Ok(metadata) => metadata.len() as usize,
        Err(error) => {
            eprintln!("Failed to get file metadata: {error}");
            exit(1);
        }
    };
    let mut json_string = String::with_capacity(json_length);
    if let Err(error) = json_file.read_to_string(&mut json_string) {
        eprintln!("Failed to read file into in memory string: {error}");
        exit(1);
    };
    let timing_read = read_timer_cpu();

    let pairs = match parse_haversine_pairs(&json_string) {
        Ok(pairs) => pairs,
        Err(message) => {
            eprintln!("Failed to parse json with error: {message}");
            exit(1);
        }
    };
    let timing_parse = read_timer_cpu();

    let average = haversine_distance_average(&pairs);
    let timing_sum = read_timer_cpu();
    let pair_count = pairs.len();

    println!(
        "
Input size: {json_length}
Pair count: {pair_count}
Average: {average}
"
    );

    let time_total = timing_sum - timing_start;
    let time_startup = timing_startup - timing_start;
    let time_read = timing_read - timing_startup;
    let time_parse = timing_parse - timing_read;
    let time_sum = timing_sum - timing_parse;

    let time_total_ms = time_total as f64 / estimate_cpu_frequency(500);
    let time_total = time_total as f64 / 100.;

    let perc_startup = time_startup as f64 / time_total;
    let perc_read = time_read as f64 / time_total;
    let perc_parse = time_parse as f64 / time_total;
    let perc_sum = time_sum as f64 / time_total;

    println!(
        "
Total time: {time_total_ms:.4}ms
Startup: {time_startup} ({perc_startup:.2}%)
read: {time_read} ({perc_read:.2}%)
parse: {time_parse} ({perc_parse:.2}%)
sum: {time_sum} ({perc_sum:.2}%)
"
    );
}

fn haversine_distance_average(pairs: &[Pair]) -> f64 {
    let mut sum = 0.;

    let sum_coeficient = 1. / pairs.len() as f64;
    for &pair in pairs {
        let Pair { x0, y0, x1, y1 } = pair;
        sum += sum_coeficient * reference_haversine(x0, y0, x1, y1, EARTH_RADIUS);
    }

    sum
}
