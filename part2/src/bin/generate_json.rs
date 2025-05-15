use std::{
    env::args,
    fs::File,
    io::{BufWriter, Write},
    ops::RangeInclusive,
    process::exit,
};

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

const Y_RANGE: RangeInclusive<f64> = -180.0..=180.0;
const X_RANGE: RangeInclusive<f64> = -90.0..=90.0;

fn main() {
    let options = process_args();
    options.print();

    let Options {
        method,
        seed,
        count,
    } = options;

    let mut rng = ChaCha20Rng::seed_from_u64(seed);
    let pairs: Vec<(Coordinate, Coordinate)> = match method {
        GenerationMethod::Uniform => generate_pairs_uniform(count, &mut rng),
        GenerationMethod::Cluster => generate_pairs_cluster(count, &mut rng),
    };

    let file = File::create("haversine_pairs.json").unwrap();
    let mut writer = BufWriter::new(file);
    let _ = writer.write("{ \"pairs\": [\n".as_bytes()).unwrap();
    for (c0, c1) in pairs[..pairs.len() - 1].iter() {
        let _ = writeln!(
            writer,
            "    {{ \"x0\": {}, \"y0\": {}, \"x1\": {}, \"y1\": {}}},",
            c0.x, c0.y, c1.x, c1.y
        )
        .unwrap();
    }
    let (c0, c1) = pairs.last().expect("pairs is not empty");
    let _ = writeln!(
        writer,
        "    {{ \"x0\": {}, \"y0\": {}, \"x1\": {}, \"y1\": {}}}
  ]
}}",
        c0.x, c0.y, c1.x, c1.y
    )
    .unwrap();
}

fn generate_pairs_uniform(count: u64, mut rng: impl Rng) -> Vec<(Coordinate, Coordinate)> {
    (0..count)
        .map(|_| Coordinate::generate_pair_uniform(&mut rng))
        .collect()
}

fn generate_pairs_cluster(_count: u64, _rng: impl Rng) -> Vec<(Coordinate, Coordinate)> {
    todo!()
}

struct Coordinate {
    x: f64,
    y: f64,
}
impl Coordinate {
    fn generate_pair_uniform(rng: &mut impl Rng) -> (Self, Self) {
        (
            Coordinate {
                x: rng.random_range(X_RANGE),
                y: rng.random_range(Y_RANGE),
            },
            Coordinate {
                x: rng.random_range(X_RANGE),
                y: rng.random_range(Y_RANGE),
            },
        )
    }
}

fn process_args() -> Options {
    let mut options = Options {
        method: GenerationMethod::Uniform,
        seed: 0,
        count: 100,
    };

    let mut args = args();
    let bin_name = args.next().expect("first argument is always binary name");

    if let Some(method) = args.next() {
        println!("{method}");
        match method.as_str() {
            "uniform" => options.method = GenerationMethod::Uniform,
            "cluster" => options.method = GenerationMethod::Cluster,
            _ => print_usage(&bin_name, 1),
        }
    }
    if let Some(seed) = args.next() {
        let seed = seed.parse().unwrap();
        options.seed = seed;
    }
    if let Some(count) = args.next() {
        let count = count.parse().unwrap();
        options.count = count;
    }

    options
}

fn print_usage(bin_name: &str, exit_code: i32) {
    eprintln!(
        "Usage: {bin_name} [uniform/cluster] [random seed] [number of point pairs to generate]"
    );
    exit(exit_code);
}

#[derive(Debug, Clone, Copy)]
enum GenerationMethod {
    Uniform,
    Cluster,
}

struct Options {
    method: GenerationMethod,
    seed: u64,
    count: u64,
}
impl Options {
    fn print(&self) {
        let Options {
            method,
            seed,
            count,
        } = self;
        println!(
            "
Method: {method:?}
Random seed: {seed}
Pair count: {count}
            "
        )
    }
}
