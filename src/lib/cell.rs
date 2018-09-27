use genome::Genome;
use super::{INFLOW_RATE_BASE, RandomGenerator};

pub struct CellPosition(pub(crate) usize, pub(crate) usize);

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

pub(crate) enum InteractionType {
    Negative,
    Positive,
}

pub struct Cell {
    pub(crate) id: CellId,
    pub(crate) parent_id: Option<CellId>,
    pub(crate) lineage: CellId,
    pub(crate) generation: usize,
    pub(crate) energy: usize,
    pub(crate) genome: Genome,
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