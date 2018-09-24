struct CellId(u32);

const PARENTLESS: CellId = CellId(0);
const POND_DEPTH: usize = 1024;
const GENOME_SIZE: usize = POND_DEPTH / 2;
const INFLOW_RATE_BASE: usize = 1000;

struct RandomIntegerGenerator([usize; 2]);
impl RandomIntegerGenerator {
    pub fn new(fseed: usize, sseed: usize) -> RandomIntegerGenerator {
        RandomIntegerGenerator([fseed, sseed])
    }

    pub fn generate(&mut self) -> usize {
        let mut x = self.0[0];
        let y = self.0[1];
        self.0[0] = y;
        x ^= x << 23;
        self.0[1] = x ^ y ^ (x >> 17) ^ (y >> 26);
        self.0[1] + y
    }
}

struct Genome([u8; GENOME_SIZE]);

impl Genome {
    pub fn new() -> Genome {
        Genome([!0; GENOME_SIZE])
    }
    pub fn random(generator: &mut RandomIntegerGenerator) -> Genome {
        let mut genome = [0; GENOME_SIZE];
        for i in 0..GENOME_SIZE {
            let mut n = generator.generate() as u8;
            genome[i] = n;
        }
        Genome(genome)
    }
}

struct Cell {
    id: CellId,
    parent_id: CellId,
    lineage: usize,
    generation: usize,
    energy: usize,
    genome: Genome,
}

impl Cell {
    pub fn new(id: CellId) -> Cell {
        Cell {
            id,
            parent_id: PARENTLESS,
            lineage: 0,
            generation: 0,
            energy: INFLOW_RATE_BASE,
            genome: Genome::new(),
        }
    }
    pub fn random(id: CellId, generator: &mut RandomIntegerGenerator) -> Cell {
        let mut res = Cell::new(id);
        res.genome = Genome::random(generator);
        res
    }
}

enum Facing {
    Up,
    Down,
    Right,
    Left,
}

struct GenomePointer {
    pub(crate) array_pointer: usize,
    pub(crate) byte_pointer: bool,
}

struct VMState {
    output_pointer: GenomePointer,
    input_pointer: GenomePointer,
    register: u8,
    output: Genome,
    facing: Facing,
    running: bool,
    loop_stack: [GenomePointer; POND_DEPTH],
    loop_stack_pointer: usize,
}

enum Instruction {
    Zero,
    Fwd,
    Back,
    Inc,
    Dec,
    ReadGenome,
    WriteGenome,
    ReadBuffer,
    WriteBuffer,
    Loop,
    Rep,
    Turn,
    Xchg(u8),
    Kill,
    Share,
}