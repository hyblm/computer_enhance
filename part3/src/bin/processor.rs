use std::{env::args, fs::File, io::Read, process::exit};

use haversine::{EARTH_RADIUS, reference_haversine};
use part3::json_parser::parse_haversine_pairs;
use part3::profile::{DropTimer, begin_profile, end_profile_and_print};
use part3::{Pair, f_name, time_function};

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

fn open_json_file() -> File {
    time_function!(1);

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

fn read_json(mut json_file: File) -> (usize, String) {
    let json_length = match json_file.metadata() {
        Ok(metadata) => metadata.len() as usize,
        Err(error) => {
            eprintln!("Failed to get file metadata: {error}");
            exit(1);
        }
    };
    time_function!(2 with json_length as u64);

    let mut json_string = String::with_capacity(json_length);
    if let Err(error) = json_file.read_to_string(&mut json_string) {
        eprintln!("Failed to read file into in memory string: {error}");
        exit(1);
    };
    (json_length, json_string)
}

fn haversine_distance_average(pairs: &[Pair]) -> f64 {
    time_function!(3 with (pairs.len() * std::mem::size_of::<Pair>()) as u64);

    let mut sum = 0.;

    let sum_coeficient = 1. / pairs.len() as f64;
    for &pair in pairs {
        let Pair { x0, y0, x1, y1 } = pair;
        sum += sum_coeficient * reference_haversine(x0, y0, x1, y1, EARTH_RADIUS);
    }

    sum
}
