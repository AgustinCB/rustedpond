extern crate rustedpond;

use rustedpond::{CellIdGenerator, CellPond, Genome, INFLOW_FREQUENCY, RandomGenerator, Statistics};
use std::time::{SystemTime, UNIX_EPOCH};

#[inline]
fn get_timestamp() -> usize {
    let start = SystemTime::now();
    start.duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .subsec_nanos() as usize
}

fn run(mut pond: CellPond,
       mut id_generator: CellIdGenerator,
       mut random_generator: RandomGenerator,
       mut statistics: Statistics) {
    loop {
        statistics.clock += 1;
        if statistics.clock % INFLOW_FREQUENCY == 0 {
            let position = &random_generator.generate_cell_position();
            pond.replace(
                position, id_generator.next(), Genome::random(&mut random_generator));
        }
    }
}

fn main() {
    let fseed = get_timestamp();
    let sseed = get_timestamp();
    let mut id_generator = CellIdGenerator::new();
    let mut random_generator = RandomGenerator::new(fseed, sseed);
    let statistics = Statistics::new();
    let pond = CellPond::new(&mut id_generator, &mut random_generator);
    run(pond, id_generator, random_generator, statistics);
    println!("Hello, world!");
}