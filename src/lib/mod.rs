mod cell;
mod cell_pond;
mod cell_vm;
mod genome;
mod instruction;
mod random_generator;
mod statistics;

const FAILED_KILL_PENALTY: usize = 1/3;
const MUTATION_RATE: usize = 5000;
const POND_HEIGHT: usize = 600;
const POND_WIDTH: usize = 800;
const POND_DEPTH: usize = 1024;
const GENOME_SIZE: usize = POND_DEPTH / 2;
const INFLOW_RATE_BASE: usize = 1000;
pub const INFLOW_FREQUENCY: usize = 100;
pub const REPORT_FREQUENCY: usize = 20000;

pub use cell::CellIdGenerator;
pub use cell_pond::CellPond;
pub use cell_vm::CellVM;
pub use genome::Genome;
pub use random_generator::RandomGenerator;
pub use statistics::Statistics;