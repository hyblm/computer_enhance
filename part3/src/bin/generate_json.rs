#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use std::{
    env::args,
    fs::File,
    io::{BufWriter, Write},
    ops::RangeInclusive,
    process::exit,
};

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

const X_EXTENT: f64 = 180.0;
const Y_EXTENT: f64 = 90.0;
const X_RANGE: RangeInclusive<f64> = -X_EXTENT..=X_EXTENT;
const Y_RANGE: RangeInclusive<f64> = -Y_EXTENT..=Y_EXTENT;

const CLUSTER_SIZE: f64 = 10.;
const X_CLUSTER_RANGE: RangeInclusive<f64> = -X_EXTENT..=X_EXTENT - CLUSTER_SIZE;
const Y_CLUSTER_RANGE: RangeInclusive<f64> = -Y_EXTENT..=Y_EXTENT - CLUSTER_SIZE;

fn main() {
    let Options {
        method,
        seed,
        count,
    } = process_args_or_bail();

    let mut rng = ChaCha20Rng::seed_from_u64(seed);
    let pairs: Vec<(Coordinate, Coordinate)> = match method {
        GenerationMethod::Uniform => generate_pairs_uniform(count, &mut rng),
        GenerationMethod::Cluster => generate_pairs_cluster(count, &mut rng),
    };
    let distances: Vec<f64> = pairs
        .iter()
        .map(|pair| {
            let (Coordinate { x: x0, y: y0 }, Coordinate { x: x1, y: y1 }) = *pair;
            let radius = haversine::EARTH_RADIUS;
            haversine::reference_haversine(x0, y0, x1, y1, radius)
        })
        .collect();

    let sum: f64 = distances.iter().sum();
    let average = sum / distances.len() as f64;
    println!(
        "
Method: {method:?}
Random seed: {seed}
Pair count: {count}

Expected Average: {average}
            "
    );

    let distances_bin: Vec<u8> = distances.into_iter().flat_map(f64::to_le_bytes).collect();

    let answers_file = File::create("haversine_distance_answers.bin").unwrap();
    let mut answers_w = BufWriter::new(answers_file);
    answers_w.write_all(&distances_bin).unwrap();

    let json_file = File::create("haversine_pairs.json").unwrap();
    let mut json_w = BufWriter::new(json_file);
    let _ = json_w.write(b"{ \"pairs\": [\n").unwrap();
    for (c0, c1) in &pairs[..pairs.len() - 1] {
        writeln!(
            json_w,
            "    {{ \"x0\": {}, \"y0\": {}, \"x1\": {}, \"y1\": {}}},",
            c0.x, c0.y, c1.x, c1.y
        )
        .unwrap();
    }
    let (c0, c1) = pairs.last().expect("pairs is not empty");
    writeln!(
        json_w,
        "    {{ \"x0\": {}, \"y0\": {}, \"x1\": {}, \"y1\": {}}}
  ]
}}",
        c0.x, c0.y, c1.x, c1.y
    )
    .unwrap();
}

fn generate_pairs_uniform(count: u64, mut rng: impl Rng) -> Vec<(Coordinate, Coordinate)> {
    (0..count)
        .map(|_| {
            (
                Coordinate::generate_pair_uniform(&mut rng),
                Coordinate::generate_pair_uniform(&mut rng),
            )
        })
        .collect()
}

const CLUSTER_COUNT: u64 = 10;
fn generate_pairs_cluster(pair_count: u64, mut rng: impl Rng) -> Vec<(Coordinate, Coordinate)> {
    let mut pairs = Vec::with_capacity(usize::try_from(pair_count).unwrap());
    let pairs_in_cluster = pair_count / CLUSTER_COUNT;

    let x_cluster_start = rng.random_range(X_CLUSTER_RANGE);
    let y_cluster_start = rng.random_range(Y_CLUSTER_RANGE);
    let mut x_cluster = x_cluster_start..=x_cluster_start + CLUSTER_SIZE;
    let mut y_cluster = y_cluster_start..=y_cluster_start + CLUSTER_SIZE;

    for i in 0..pair_count {
        if i % pairs_in_cluster == 0 {
            let x_cluster_start = rng.random_range(X_CLUSTER_RANGE);
            let y_cluster_start = rng.random_range(Y_CLUSTER_RANGE);
            x_cluster = x_cluster_start..=x_cluster_start + CLUSTER_SIZE;
            y_cluster = y_cluster_start..=y_cluster_start + CLUSTER_SIZE;
        }

        pairs.push((
            Coordinate::generate_pair_cluster(&mut rng, x_cluster.clone(), y_cluster.clone()),
            Coordinate::generate_pair_cluster(&mut rng, x_cluster.clone(), y_cluster.clone()),
        ));
    }
    pairs
}

struct Coordinate {
    x: f64,
    y: f64,
}
impl Coordinate {
    fn generate_pair_uniform(rng: &mut impl Rng) -> Self {
        Self {
            x: rng.random_range(X_RANGE),
            y: rng.random_range(Y_RANGE),
        }
    }

    fn generate_pair_cluster(
        rng: &mut impl Rng,
        x_cluster: RangeInclusive<f64>,
        y_cluster: RangeInclusive<f64>,
    ) -> Self {
        Self {
            x: rng.random_range(x_cluster),
            y: rng.random_range(y_cluster),
        }
    }
}

fn process_args_or_bail() -> Options {
    let mut options = Options {
        method: GenerationMethod::Uniform,
        seed: 0,
        count: 100,
    };

    let mut args = args();
    let bin_name = args.next().expect("first argument is always binary name");

    if let Err(message) = process_options(args, &mut options) {
        eprintln!("Error: {message}\n");
        {
            let bin_name: &str = &bin_name;
            eprintln!(
                "Usage: {bin_name} [uniform/cluster] [random seed] [number of point pairs to generate]"
            );
        };
        exit(1);
    }

    options
}

fn process_options(mut args: std::env::Args, options: &mut Options) -> Result<(), String> {
    if let Some(method) = args.next() {
        match method.as_str() {
            "uniform" => options.method = GenerationMethod::Uniform,
            "cluster" => options.method = GenerationMethod::Cluster,
            _ => return Err(format!("Generation method can only be \"uniform\" or \"cluster\". Received \"{method}\" instead.")),
        }
    }
    if let Some(seed) = args.next() {
        let Ok(seed) = seed.parse() else {
            return Err(format!("{seed} is not a 64 bit represantable integer"));
        };
        options.seed = seed;
    }
    if let Some(count) = args.next() {
        let Ok(count) = count.parse() else {
            return Err(format!("{count} is not a 64 bit represantable integer"));
        };
        options.count = count;
    }

    Ok(())
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
