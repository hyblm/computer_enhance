use std::{env::args, fs::File, io::Read, process::exit};

use haversine::{reference_haversine, EARTH_RADIUS};
use part2::json_parser::parse_haversine_pairs;
use part2::Pair;

fn main() {
    let Some(path) = args().nth(1) else {
        eprintln!("No file recieved");
        exit(1);
    };

    let mut json_file = match File::open(path) {
        Ok(file) => file,
        Err(error) => {
            eprintln!("Failed to open file with error: {error}");
            exit(1);
        }
    };
    let json_length = json_file.metadata().unwrap().len() as usize;
    let mut json_string = String::with_capacity(json_length);
    json_file.read_to_string(&mut json_string).unwrap();

    let pairs = match parse_haversine_pairs(&json_string) {
        Ok(pairs) => pairs,
        Err(message) => {
            eprintln!("Failed to parse json with error: {message}");
            exit(1);
        }
    };

    let average = haversine_distance_average(&pairs);
    println!("Average: {average}");
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
