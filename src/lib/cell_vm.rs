use cell::{CellIdGenerator, CellPosition, InteractionType};
use cell_pond::CellPond;
use instruction::Instruction;
use genome::{Genome, GenomePointer};
use random_generator::RandomGenerator;
use statistics::Statistics;
use super::{FAILED_KILL_PENALTY, MUTATION_RATE, POND_DEPTH};

pub(crate) enum Facing {
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

pub struct CellVM<'a> {
    pond: &'a mut CellPond,
    id_generator: &'a mut CellIdGenerator,
    random_generator: &'a mut RandomGenerator,
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

impl<'a> CellVM<'a> {
    pub fn new(cell: CellPosition,
               pond: &'a mut CellPond,
               id_generator: &'a mut CellIdGenerator,
               random_generator: &'a mut RandomGenerator,
               statistics: &'a mut Statistics) -> CellVM<'a> {
        CellVM {
            pond,
            id_generator,
            random_generator,
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
        self.statistics.cell_executions += 1;
        while self.pond.cell(&self.cell).energy > 0 && self.running {
            self.maybe_mutate();
            self.pond.cell(&self.cell).energy -= 1;
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
        }
        self.maybe_reproduce();
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
                self.register = (self.register + 1) & 0x0f;
            },
            Instruction::Dec => {
                self.register = (self.register.wrapping_sub(1)) & 0x0f;
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
                    {
                        let neighbor = self.pond.get_neighbor(&self.cell, &self.facing);
                        if neighbor.generation > 2 {
                            self.statistics.viable_cell_shares += 1;
                        }
                        neighbor.energy = neighbor_energy;
                    }
                    self.pond.cell(&self.cell).energy = cell_energy;
                }
            },
            Instruction::Kill => {
                if self.can_access_neighbor(InteractionType::Negative) {
                    let neighbor = self.pond.get_neighbor(&self.cell, &self.facing);
                    if neighbor.generation > 2 {
                        self.statistics.viable_cells_killed += 1;
                    }
                    neighbor.id = self.id_generator.next();
                    neighbor.genome.0[0] = !0;
                    neighbor.genome.0[1] = !0;
                    neighbor.parent_id = None;
                    neighbor.lineage = neighbor.id.clone();
                    neighbor.generation = 0;
                } else {
                    let cell_energy = self.pond.cell(&self.cell).energy;
                    let penalty = cell_energy * FAILED_KILL_PENALTY;
                    self.pond.cell(&self.cell).energy = if cell_energy <= penalty {
                        0
                    } else {
                        cell_energy - penalty
                    };
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
                self.register, interaction, self.random_generator.generate_integer() as u8)
    }

    #[inline]
    fn maybe_mutate(&mut self) {
        if self.random_generator.generate_integer() < MUTATION_RATE {
            let new_instruction = self.random_generator.generate_integer() as u8  & 0x0f;
            if self.random_generator.generate_boolean() {
                self.pond
                    .cell(&self.cell).genome
                    .set(&self.input_pointer, new_instruction);
            } else {
                self.register = new_instruction;
            }
        }
    }

    #[inline]
    pub fn maybe_reproduce(&mut self) {
        let neighbor_energy =
            self.pond.get_neighbor(&self.cell, &self.facing).energy;
        if neighbor_energy > 0 &&
            self.output.0[0] != 0xff &&
            self.can_access_neighbor(InteractionType::Negative) {
            let parent = self.pond.cell(&self.cell).id.clone();
            let lineage = self.pond.cell(&self.cell).lineage.clone();
            let generation = self.pond.cell(&self.cell).generation + 1;
            let neighbor = self.pond.get_neighbor(&self.cell, &self.facing);
            if neighbor.generation > 2 {
                self.statistics.viable_cell_replaced += 1;
            }
            neighbor.id = self.id_generator.next();
            neighbor.parent_id = Some(parent);
            neighbor.lineage = lineage;
            neighbor.generation = generation;
            neighbor.genome = self.output.clone();
        }
    }
}