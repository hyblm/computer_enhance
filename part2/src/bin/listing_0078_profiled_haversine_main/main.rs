pub mod listing_0076_simple_profiler;
mod listing_0077_profiled_lookup_json_parser;

use std::{env::args, fs::File, io::Read, process::exit};

use haversine::{reference_haversine, EARTH_RADIUS};
use listing_0076_simple_profiler::{begin_profile, end_profile_and_print, DropTimer};
use listing_0077_profiled_lookup_json_parser::parse_haversine_pairs;
use part2::Pair;

fn main() {
    begin_profile();

    let json_file = open_json_file();
    let (json_length, json_string) = read_json(json_file);

    let pairs = match parse_haversine_pairs(&json_string) {
        Ok(pairs) => pairs,
        Err(message) => {
            eprintln!("Failed to parse json with error: {message}");
            exit(1);
        }
    };

    let average = haversine_distance_average(&pairs);
    let pair_count = pairs.len();

    println!(
        "
Input size: {json_length}
Pair count: {pair_count}
Average: {average}
"
    );

    end_profile_and_print();
}

fn read_json(mut json_file: File) -> (usize, String) {
    time_function!(1);

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
    (json_length, json_string)
}

fn open_json_file() -> File {
    time_function!(0);

    let Some(path) = args().nth(1) else {
        eprintln!("No file recieved");
        exit(1);
    };

    let json_file = match File::open(path) {
        Ok(file) => file,
        Err(error) => {
            eprintln!("Failed to open file with error: {error}");
            exit(1);
        }
    };
    json_file
}

fn haversine_distance_average(pairs: &[Pair]) -> f64 {
    time_function!(3);

    let mut sum = 0.;

    if !pairs.is_empty() {
        let sum_coeficient = 1. / pairs.len() as f64;
        for &pair in pairs {
            let Pair { x0, y0, x1, y1 } = pair;
            sum += sum_coeficient * reference_haversine(x0, y0, x1, y1, EARTH_RADIUS);
        }
    }

    sum
}
