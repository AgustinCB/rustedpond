use std::fmt;
use std::ops::{Index, IndexMut};
use instruction::Instruction;

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
    pub(crate) instruction_executions: InstructionCounter,
    pub(crate) cell_executions: usize,
    pub clock: usize,
    pub(crate) viable_cells_killed: usize,
    pub(crate) viable_cell_shares: usize,
    pub(crate) viable_cell_replaced: usize,
}

impl Statistics {
    pub fn new() -> Statistics {
        Statistics {
            instruction_executions: InstructionCounter::new(),
            cell_executions: 0,
            clock: 0,
            viable_cells_killed: 0,
            viable_cell_shares: 0,
            viable_cell_replaced: 0,
        }
    }

    #[inline]
    pub fn metabolism(&self) -> usize {
        if self.cell_executions == 0 {
            0
        } else {
            self.total_metabolism() / self.cell_executions
        }
    }

    #[inline]
    fn total_metabolism(&self) -> usize {
        Instruction::iterator()
            .fold(0, |sum, i| sum + self.instruction_executions[i])
    }
}

impl fmt::Display for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{},{},{}",
               self.viable_cell_replaced,
               self.viable_cells_killed,
               self.viable_cell_shares)?;
        for instruction in Instruction::iterator() {
            let value = if self.cell_executions == 0 {
                0.0
            } else {
                self.instruction_executions[instruction] as f64 / self.cell_executions as f64
            };
            write!(f, "{:04},", value)?;
        }
        Ok(())
    }
}