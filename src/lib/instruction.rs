use std::slice::Iter;

#[derive(PartialEq)]
pub(crate) enum Instruction {
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

impl Instruction {
    pub(crate) fn iterator() -> Iter<'static, Instruction> {
        static INSTRUCTIONS: [Instruction; 16] = [
            Instruction::Zero,
            Instruction::Fwd,
            Instruction::Back,
            Instruction::Inc,
            Instruction::Dec,
            Instruction::ReadGenome,
            Instruction::WriteGenome,
            Instruction::ReadBuffer,
            Instruction::WriteBuffer,
            Instruction::Loop,
            Instruction::Rep,
            Instruction::Turn,
            Instruction::Xchg,
            Instruction::Kill,
            Instruction::Share,
            Instruction::Stop,
        ];
        INSTRUCTIONS.into_iter()
    }
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