use cell::{Cell, CellId, CellIdGenerator, CellPosition};
use cell_vm::Facing;
use genome::Genome;
use random_generator::RandomGenerator;
use super::{INFLOW_RATE_BASE, POND_HEIGHT, POND_WIDTH};

pub struct CellPond {
    grind: Vec<Vec<Cell>>,
}

impl CellPond {
    pub fn new(id_generator: &mut CellIdGenerator, generator: &mut RandomGenerator) -> CellPond {
        let mut grind = Vec::with_capacity(POND_WIDTH);
        for i in 0..POND_WIDTH {
            grind[i] = Vec::with_capacity(POND_HEIGHT);
            for j in 0..POND_HEIGHT {
                grind[i][j] = Cell::random(id_generator, generator);
            }
        }
        CellPond {
            grind,
        }
    }

    #[inline]
    pub fn replace(&mut self, position: &CellPosition, new_id: CellId, genome: Genome) {
        let cell = &mut self.grind[position.0][position.1];
        cell.id = new_id.clone();
        cell.parent_id = None;
        cell.lineage = new_id;
        cell.generation = 0;
        cell.energy = INFLOW_RATE_BASE;
        cell.genome = genome;
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

    #[inline]
    pub fn total_energy(&self) -> usize {
        self.perform_on_active(0, |acc, cell| cell.energy + acc)
    }

    #[inline]
    pub fn total_active_cells(&self) -> usize {
        self.perform_on_active(0, |acc, _| acc+1)
    }

    #[inline]
    pub fn total_viable_replicators(&self) -> usize {
        self.perform_on_active(
            0, |acc, cell| if cell.generation > 2 { acc + 1 } else { acc })
    }

    #[inline]
    pub fn max_generation(&self) -> usize {
        self.perform_on_active(0, |g, cell|
            if cell.generation > g {
                cell.generation
            } else {
                g
            })
    }

    fn perform_on_active<R, T: Fn(R, &Cell) -> R>(&self, zero: R, op: T) -> R {
        let mut acc = zero;
        for x in 0..POND_WIDTH {
            for y in 0..POND_HEIGHT {
                let c = &self.grind[x][y];
                if c.energy > 0 {
                    acc = op(acc, c);
                }
            }
        }
        acc
    }
}