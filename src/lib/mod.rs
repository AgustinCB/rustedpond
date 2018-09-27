use std::ops::{Index, IndexMut};

const FAILED_KILL_PENALTY: usize = 1/3;
const MUTATION_RATE: usize = 5000;
const POND_HEIGHT: usize = 600;
const POND_WIDTH: usize = 800;
const POND_DEPTH: usize = 1024;
const GENOME_SIZE: usize = POND_DEPTH / 2;
const INFLOW_RATE_BASE: usize = 1000;

pub(crate) struct InstructionCounter {
    zero: usize,
    fwd: usize,
    back: usize,
    inc: usize,
    dec: usize,
    read_genome: usize,
    write_genome: usize,
    read_buffer: usize,
    write_buffer: usize,
    loop_instruction: usize,
    rep: usize,
    turn: usize,
    xchg: usize,
    kill: usize,
    share: usize,
    stop: usize,
}

impl InstructionCounter {
    pub(crate) fn new() -> InstructionCounter {
        InstructionCounter {
            zero: 0,
            fwd: 0,
            back: 0,
            inc: 0,
            dec: 0,
            read_genome: 0,
            write_genome: 0,
            read_buffer: 0,
            write_buffer: 0,
            loop_instruction: 0,
            rep: 0,
            turn: 0,
            xchg: 0,
            kill: 0,
            share: 0,
            stop: 0,
        }
    }
}

impl<'a> Index<&'a Instruction> for InstructionCounter {
    type Output = usize;
    fn index(&self, instruction: &'a Instruction) -> &usize {
        match instruction {
            Instruction::Zero => &self.zero,
            Instruction::Fwd => &self.fwd,
            Instruction::Back => &self.back,
            Instruction::Inc => &self.inc,
            Instruction::Dec => &self.dec,
            Instruction::ReadGenome => &self.read_genome,
            Instruction::WriteGenome => &self.write_genome,
            Instruction::ReadBuffer => &self.read_buffer,
            Instruction::WriteBuffer => &self.write_buffer,
            Instruction::Loop => &self.loop_instruction,
            Instruction::Rep => &self.rep,
            Instruction::Turn => &self.turn,
            Instruction::Xchg => &self.xchg,
            Instruction::Kill => &self.kill,
            Instruction::Share => &self.share,
            Instruction::Stop => &self.stop,
        }
    }
}

impl<'a> IndexMut<&'a Instruction> for InstructionCounter {
    fn index_mut(&mut self, instruction: &'a Instruction) -> &mut usize {
        match instruction {
            Instruction::Zero => &mut self.zero,
            Instruction::Fwd => &mut self.fwd,
            Instruction::Back => &mut self.back,
            Instruction::Inc => &mut self.inc,
            Instruction::Dec => &mut self.dec,
            Instruction::ReadGenome => &mut self.read_genome,
            Instruction::WriteGenome => &mut self.write_genome,
            Instruction::ReadBuffer => &mut self.read_buffer,
            Instruction::WriteBuffer => &mut self.write_buffer,
            Instruction::Loop => &mut self.loop_instruction,
            Instruction::Rep => &mut self.rep,
            Instruction::Turn => &mut self.turn,
            Instruction::Xchg => &mut self.xchg,
            Instruction::Kill => &mut self.kill,
            Instruction::Share => &mut self.share,
            Instruction::Stop => &mut self.stop,
        }
    }
}
pub struct Statistics {
    instruction_executions: InstructionCounter,
}

impl Statistics {
    pub fn new() -> Statistics {
        Statistics {
            instruction_executions: InstructionCounter::new(),
        }
    }
}

pub struct CellPosition(usize, usize);

#[derive(Clone, PartialEq)]
pub struct CellId(usize);

pub struct CellIdGenerator {
    current: usize,
}

impl CellIdGenerator {
    #[inline]
    pub fn new() -> CellIdGenerator {
        CellIdGenerator {
            current: 0,
        }
    }

    #[inline]
    pub fn next(&mut self) -> CellId {
        self.current += 1;
        CellId(self.current)
    }
}

pub struct RandomGenerator([usize; 2]);
impl RandomGenerator {
    #[inline]
    pub fn new(fseed: usize, sseed: usize) -> RandomGenerator {
        RandomGenerator([fseed, sseed])
    }

    #[inline]
    pub fn generate_integer(&mut self) -> usize {
        let mut x = self.0[0];
        let y = self.0[1];
        self.0[0] = y;
        x ^= x << 23;
        self.0[1] = x ^ y ^ (x >> 17) ^ (y >> 26);
        self.0[1] + y
    }

    #[inline]
    pub fn generate_boolean(&mut self) -> bool {
        (self.generate_integer() & 0x80) > 0
    }
}

struct Genome(pub(crate) [u8; GENOME_SIZE]);

impl Genome {
    #[inline]
    pub fn new() -> Genome {
        Genome([!0; GENOME_SIZE])
    }
    #[inline]
    pub fn random(generator: &mut RandomGenerator) -> Genome {
        let mut genome = [0; GENOME_SIZE];
        for i in 0..GENOME_SIZE {
            let mut n = generator.generate_integer() as u8;
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

enum InteractionType {
    Negative,
    Positive,
}

pub struct Cell {
    id: CellId,
    parent_id: Option<CellId>,
    lineage: CellId,
    generation: usize,
    energy: usize,
    genome: Genome,
}

impl Cell {
    #[inline]
    pub fn new(generator: &mut CellIdGenerator) -> Cell {
        Cell {
            id: generator.next(),
            parent_id: None,
            lineage: CellId(0),
            generation: 0,
            energy: INFLOW_RATE_BASE,
            genome: Genome::new(),
        }
    }
    #[inline]
    pub fn random(
        id_generator: &mut CellIdGenerator, generator: &mut RandomGenerator) -> Cell {
        let mut res = Cell::new(id_generator);
        res.genome = Genome::random(generator);
        res
    }

    #[inline]
    pub(crate) fn can_be_accessed(
        &self, guess: u8, interaction: InteractionType, threshold: u8) -> bool {
        match interaction {
            InteractionType::Positive => {
                self.parent_id == None ||
                    (threshold & 0x0f) >=
                        ((self.genome.0[0] & 0x0f) ^ (guess & 0x0f)).count_ones() as u8
            },
            InteractionType::Negative => {
                self.parent_id == None ||
                    (threshold & 0x0f) <=
                        ((self.genome.0[0] & 0x0f) ^ (guess & 0x0f)).count_ones() as u8
            }
        }
    }
}

type CellGrind = [[Cell; POND_HEIGHT]; POND_WIDTH];
pub struct CellPond {
    grind: CellGrind,
}

impl CellPond {
    pub fn new(grind: CellGrind) -> CellPond {
        CellPond {
            grind,
        }
    }

    #[inline]
    pub(crate) fn cell(&mut self, position: &CellPosition) -> &mut Cell {
        &mut self.grind[position.0][position.1]
    }

    #[inline]
    pub(crate) fn get_neighbor(&mut self, position: &CellPosition, facing: &Facing) -> &mut Cell {
        match facing {
            Facing::Left => {
                let x = if position.0 == 0 {
                    POND_WIDTH-1
                } else {
                    position.0
                };
                &mut self.grind[x][position.1]
            },
            Facing::Right => {
                let x = (position.0 + 1) % POND_WIDTH;
                &mut self.grind[x][position.1]
            },
            Facing::Up => {
                let y = (position.1 + 1) % POND_HEIGHT;
                &mut self.grind[position.0][y]
            },
            Facing::Down => {
                let y = if position.1 == 0 {
                    POND_HEIGHT-1
                } else {
                    position.1-1
                };
                &mut self.grind[position.0][y]
            },
        }
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
            self.array_pointer = if self.array_pointer == 0 {
                GENOME_SIZE-1
            } else {
                self.array_pointer-1
            }
        }
        self.is_lower_byte = !self.is_lower_byte;
    }
}

pub struct VMState<'a> {
    pond: &'a mut CellPond,
    id_generator: &'a mut CellIdGenerator,
    number_generator: &'a mut RandomGenerator,
    statistics: &'a mut Statistics,
    cell: CellPosition,
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
    pub fn new(cell: CellPosition,
               pond: &'a mut CellPond,
               id_generator: &'a mut CellIdGenerator,
               number_generator: &'a mut RandomGenerator,
               statistics: &'a mut Statistics) -> VMState<'a> {
        VMState {
            pond,
            id_generator,
            number_generator,
            statistics,
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
        while self.pond.cell(&self.cell).energy > 0 && self.running {
            self.maybe_mutate();
            let instruction_byte = self.pond.cell(&self.cell).genome.get(&self.input_pointer);
            self.input_pointer.next();
            let instruction = Instruction::from(instruction_byte);
            self.statistics.instruction_executions[&instruction] += 1;
            if self.loop_stack_depth == 0 {
                self.execute_instruction(instruction);
            } else if instruction == Instruction::Loop {
                self.loop_stack_depth += 1;
            } else if instruction == Instruction::Rep {
                self.loop_stack_depth -= 1;
            }
            self.pond.cell(&self.cell).energy -= 1;
        }
    }

    #[inline]
    fn execute_instruction(
        &mut self, instruction: Instruction) {
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
                self.register = self.pond.cell(&self.cell).genome.get(&self.input_pointer);
            },
            Instruction::WriteGenome => {
                self.pond.cell(&self.cell).genome.set(&self.input_pointer, self.register);
            },
            Instruction::ReadBuffer => {
                self.register = self.output.get(&self.output_pointer);
            },
            Instruction::WriteBuffer => {
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
                self.register = self.pond.cell(&self.cell).genome.get(&self.input_pointer);
                self.pond.cell(&self.cell).genome.set(&self.input_pointer, register);
                self.input_pointer.next();
            },
            Instruction::Share => {
                if self.can_access_neighbor(InteractionType::Positive) {
                    let total_energy = self.pond.cell(&self.cell).energy +
                        self.pond.get_neighbor(&self.cell, &self.facing).energy;
                    let neighbor_energy = total_energy/2;
                    let cell_energy = total_energy - neighbor_energy;
                    self.pond.get_neighbor(&self.cell, &self.facing).energy = neighbor_energy;
                    self.pond.cell(&self.cell).energy = cell_energy;
                }
            },
            Instruction::Kill => {
                if self.can_access_neighbor(InteractionType::Negative) {
                    let neighbor = self.pond.get_neighbor(
                        &self.cell, &self.facing);
                    neighbor.id = self.id_generator.next();
                    neighbor.genome.0[0] = !0;
                    neighbor.genome.0[1] = !0;
                    neighbor.parent_id = None;
                    neighbor.lineage = neighbor.id.clone();
                    neighbor.generation = 0;
                } else {
                    let cell_energy = self.pond.cell(&self.cell).energy;
                    self.pond.cell(&self.cell).energy
                        .wrapping_sub(cell_energy * FAILED_KILL_PENALTY);
                }
            },
            Instruction::Stop => {
                self.running = false;
            },
        }
    }

    #[inline]
    fn can_access_neighbor(&mut self, interaction: InteractionType) -> bool {
        self.pond.get_neighbor(&self.cell, &self.facing)
            .can_be_accessed(
                self.register, interaction, self.number_generator.generate_integer() as u8)
    }

    #[inline]
    fn maybe_mutate(&mut self) {
        if self.number_generator.generate_integer() < MUTATION_RATE {
            let new_instruction = self.number_generator.generate_integer() as u8  & 0x0f;
            if self.number_generator.generate_boolean() {
                self.pond
                    .cell(&self.cell).genome
                    .set(&self.input_pointer, new_instruction);
            } else {
                self.register = new_instruction;
            }
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