extern crate rustedpond;

use rustedpond::*;
use std::time::{SystemTime, UNIX_EPOCH};

#[inline]
fn get_timestamp() -> usize {
    let start = SystemTime::now();
    start.duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .subsec_nanos() as usize
}

#[inline]
fn mutate_cell(pond: &mut CellPond,
               random_generator: &mut RandomGenerator,
               id_generator: &mut CellIdGenerator) {
    let position = &random_generator.generate_cell_position();
    pond.replace(
        position, id_generator.next(), Genome::random(random_generator));
}

#[inline]
fn execute_cell(pond: &mut CellPond,
                random_generator: &mut RandomGenerator,
                id_generator: &mut CellIdGenerator,
                statistics: &mut Statistics) {
    let position = random_generator.generate_cell_position();
    let mut vm = CellVM::new(position, pond, id_generator, random_generator, statistics);
    vm.execute();
}

#[inline]
fn do_report(pond: &CellPond, statistics: &mut Statistics) {
    println!("{},{},{},{},{},{},{}",
           statistics.clock,
           pond.total_energy(),
           pond.total_active_cells(),
           pond.total_viable_replicators(),
           pond.max_generation(),
           statistics,
           statistics.metabolism());
    statistics.zero();
}

fn run(mut pond: CellPond,
       mut id_generator: CellIdGenerator,
       mut random_generator: RandomGenerator,
       mut statistics: Statistics) {
    loop {
        statistics.clock += 1;
        if statistics.clock % REPORT_FREQUENCY == 0 {
            do_report(&pond, &mut statistics);
        }
        if statistics.clock % INFLOW_FREQUENCY == 0 {
            mutate_cell(&mut pond, &mut random_generator, &mut id_generator);
        }
        execute_cell(&mut pond, &mut random_generator, &mut id_generator, &mut statistics);
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