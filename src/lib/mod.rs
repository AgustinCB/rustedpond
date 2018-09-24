use std::cmp::min;

struct CellId(u32);

const PARENTLESS: CellId = CellId(0);
const POND_DEPTH: usize = 1024;
const GENOME_SIZE: usize = POND_DEPTH / 2;
const INFLOW_RATE_BASE: usize = 1000;

struct RandomIntegerGenerator([usize; 2]);
impl RandomIntegerGenerator {
    #[inline]
    pub fn new(fseed: usize, sseed: usize) -> RandomIntegerGenerator {
        RandomIntegerGenerator([fseed, sseed])
    }

    #[inline]
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
    #[inline]
    pub fn new() -> Genome {
        Genome([!0; GENOME_SIZE])
    }
    #[inline]
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
    #[inline]
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
    #[inline]
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
    pub(crate) is_lower_byte: bool,
}

impl GenomePointer {
    #[inline]
    pub(crate) fn new(array_pointer: usize, is_lower_byte: bool) -> GenomePointer {
        GenomePointer { array_pointer, is_lower_byte }
    }

    #[inline]
    pub(crate) fn get(&self, genome: &Genome) -> u8 {
        if self.is_lower_byte {
            (genome.0[self.array_pointer] & 0xf) as u8
        } else {
            ((genome.0[self.array_pointer] >> 4) & 0xf) as u8
        }
    }

    #[inline]
    pub(crate) fn next(&mut self) {
        if !self.is_lower_byte {
            self.array_pointer = (self.array_pointer + 1) % GENOME_SIZE;
        }
        self.is_lower_byte = !self.is_lower_byte;
    }

    #[inline]
    pub(crate) fn prev(&mut self) {
        if self.is_lower_byte {
            self.array_pointer =
                (((self.array_pointer - 1) % GENOME_SIZE) + GENOME_SIZE) % GENOME_SIZE;
        }
        self.is_lower_byte = !self.is_lower_byte;
    }
}

struct VMState<'a> {
    output_pointer: GenomePointer,
    input_pointer: GenomePointer,
    register: u8,
    output: Genome,
    input: &'a Genome,
    facing: Facing,
    running: bool,
    loop_stack: Vec<GenomePointer>,
    loop_stack_pointer: usize,
}

impl<'a> VMState<'a> {
    pub fn new(input: &'a Genome) -> VMState<'a> {
        VMState {
            input,
            output_pointer: GenomePointer::new(0, true),
            input_pointer: GenomePointer::new(0, true),
            register: 0,
            output: Genome::new(),
            facing: Facing::Left,
            running: true,
            loop_stack: Vec::with_capacity(POND_DEPTH),
            loop_stack_pointer: 0,
        }
    }

    pub fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Zero => {
                self.output_pointer.array_pointer = 0;
                self.output_pointer.is_lower_byte = true;
                self.facing = Facing::Left;
                self.register = 0;
            },
            Instruction::Fwd => self.output_pointer.next(),
            Instruction::Back => self.output_pointer.prev(),
            Instruction::Inc => {
                self.register= (self.register+ 1) & 0x0f;
            },
            Instruction::Dec => {
                self.register= (self.register- 1) & 0x0f;
            },
            Instruction::ReadGenome => {
                self.register= self.output_pointer.get(self.input);
            },
            _ => panic!("Not implemented yet"),
        }
    }
}

pub enum Instruction {
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