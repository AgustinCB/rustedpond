use std::cmp::min;

pub struct CellId(u32);

const PARENTLESS: CellId = CellId(0);
const POND_DEPTH: usize = 1024;
const GENOME_SIZE: usize = POND_DEPTH / 2;
const INFLOW_RATE_BASE: usize = 1000;

pub struct RandomIntegerGenerator([usize; 2]);
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

    #[inline]
    pub(crate) fn get(&self, pointer: &GenomePointer) -> u8 {
        if pointer.is_lower_byte {
            (self.0[pointer.array_pointer] & 0xf) as u8
        } else {
            ((self.0[pointer.array_pointer] >> 4) & 0xf) as u8
        }
    }

    #[inline]
    pub(crate) fn set(&mut self, pointer: &GenomePointer, value: u8) {
        self.0[pointer.array_pointer] = if pointer.is_lower_byte {
            let high_bits = self.0[pointer.array_pointer] & 0xf0;
            high_bits | (value & 0x0f)
        } else {
            let low_bits = self.0[pointer.array_pointer] & 0x0f;
            ((value & 0x0f) << 4) | low_bits
        }
    }
}

pub struct Cell {
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

impl From<u8> for Facing {
    fn from(byte: u8) -> Facing {
        match byte & 0x3 {
            0x0 => Facing::Left,
            0x1 => Facing::Right,
            0x2 => Facing::Up,
            0x3 => Facing::Down,
            _ => panic!("Can't happen"),
        }
    }
}

#[derive(Clone)]
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

pub struct VMState<'a> {
    cell: &'a mut Cell,
    output_pointer: GenomePointer,
    input_pointer: GenomePointer,
    register: u8,
    output: Genome,
    facing: Facing,
    running: bool,
    loop_stack: Vec<GenomePointer>,
    loop_stack_depth: usize,
}

impl<'a> VMState<'a> {
    pub fn new(cell: &'a mut Cell) -> VMState<'a> {
        VMState {
            cell,
            output_pointer: GenomePointer::new(0, true),
            input_pointer: GenomePointer::new(0, true),
            register: 0,
            output: Genome::new(),
            facing: Facing::Left,
            running: true,
            loop_stack: Vec::with_capacity(POND_DEPTH),
            loop_stack_depth: 0,
        }
    }

    pub fn execute(&mut self) {
        while self.cell.energy > 0 && self.running {
            let instruction_byte = self.cell.genome.get(&self.input_pointer);
            self.input_pointer.next();
            let instruction = Instruction::from(instruction_byte);
            if self.loop_stack_depth == 0 {
                self.execute_instruction(instruction);
            } else if instruction == Instruction::Loop {
                self.loop_stack_depth += 1;
            } else if instruction == Instruction::Rep {
                self.loop_stack_depth -= 1;
            }
            self.cell.energy -= 1;
        }
    }

    #[inline]
    fn execute_instruction(&mut self, instruction: Instruction) {
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
                self.register = (self.register+ 1) & 0x0f;
            },
            Instruction::Dec => {
                self.register = (self.register- 1) & 0x0f;
            },
            Instruction::ReadGenome => {
                self.register = self.cell.genome.get(&self.input_pointer);
            },
            Instruction::WriteGenome => {
                self.cell.genome.set(&self.input_pointer, self.register);
            },
            Instruction::ReadGenome => {
                self.register = self.output.get(&self.output_pointer);
            },
            Instruction::WriteGenome => {
                self.output.set(&self.output_pointer, self.register);
            },
            Instruction::Loop => {
                if self.register == 0 {
                    self.loop_stack_depth = 1;
                } else {
                    self.loop_stack.push(self.input_pointer.clone());
                }
            },
            Instruction::Rep => {
                if let Some(mut input_pointer) = self.loop_stack.pop() {
                    if self.register > 0 {
                        self.input_pointer = input_pointer;
                    }
                }
            },
            Instruction::Turn => {
                self.facing = Facing::from(self.register);
            },
            Instruction::Xchg => {
                let register = self.register;
                self.register = self.cell.genome.get(&self.input_pointer);
                self.cell.genome.set(&self.input_pointer, register);
                self.input_pointer.next();
            }
            Instruction::Stop => {
                self.running = false;
            },
            _ => panic!("Not implemented yet"),
        }
    }
}

#[derive(PartialEq)]
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
    Xchg,
    Kill,
    Share,
    Stop,
}

impl From<u8> for Instruction {
    fn from(instruction: u8) -> Self {
        match instruction & 0x0f {
            0x0 => Instruction::Zero,
            0x1 => Instruction::Fwd,
            0x2 => Instruction::Back,
            0x3 => Instruction::Inc,
            0x4 => Instruction::Dec,
            0x5 => Instruction::ReadGenome,
            0x6 => Instruction::WriteGenome,
            0x7 => Instruction::ReadBuffer,
            0x8 => Instruction::WriteBuffer,
            0x9 => Instruction::Loop,
            0xa => Instruction::Rep,
            0xb => Instruction::Turn,
            0xc => Instruction::Xchg,
            0xd => Instruction::Kill,
            0xe => Instruction::Share,
            0xf => Instruction::Stop,
            _ => panic!("Can't happen"),
        }
    }
}