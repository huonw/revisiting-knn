extern crate simple_parallel;
extern crate crossbeam;

use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

struct LabelPixel {
    label: i64,
    pixels: Vec<i64>,
}

fn slurp_file(file: &str) -> Vec<LabelPixel> {
    BufReader::new(File::open(file).unwrap())
        .lines()
        .skip(1)
        .map(|line| {
            let line = line.unwrap();
            let mut splits = line.trim().split(',').map(|s| s.parse().unwrap());

            let label = splits.next().unwrap();

            let pixels = splits.collect();

            LabelPixel {
                label: label,
                pixels: pixels,
            }
        })
        .collect()
}

fn distance_sqr(x: &[i64], y: &[i64]) -> i64 {
    x.iter()
        .zip(y.iter())
        .fold(0, |s, (&a, &b)| s + (a - b) * (a - b))
}

fn classify(training: &[LabelPixel], pixels: &[i64]) -> i64 {
    let mut min = 0x7FFF_FFFF;
    let mut min_pixel = None;

    for p in training {
        let d = distance_sqr(&p.pixels, pixels);
        if d < min {
            min = d;
            min_pixel = Some(p);
        }
    }

    min_pixel.unwrap().label
}

#[cfg(feature = "sequential")]
fn main() {
    let training_set = slurp_file("trainingsample.csv");
    let validation_sample = slurp_file("validationsample.csv");

    let num_correct = validation_sample.iter()
        .filter(|x| classify(&training_set, &x.pixels) == x.label)
        .count();

    println!("Percentage correct: {:.1}%",
             num_correct as f64 / validation_sample.len() as f64 * 100.0);
}

#[cfg(not(feature = "sequential"))]
fn main() {
    // load files
    let training_set = slurp_file("trainingsample.csv");
    let validation_sample = slurp_file("validationsample.csv");

    // create a thread pool
    let mut pool = simple_parallel::Pool::new(4);

    crossbeam::scope(|scope| {
        let num_correct =
            pool.unordered_map(scope, &validation_sample, |x| {
                // is it classified right? (in parallel)
                classify(&training_set, &x.pixels) == x.label
            })
            .filter(|t| t.1)
            .count();

        println!("Percentage correct: {:.1}%",
                 num_correct as f64 / validation_sample.len() as f64 * 100.0);
    });
}
