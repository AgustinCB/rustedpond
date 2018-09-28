use cell::{Cell, CellId, CellPosition};
use cell_vm::Facing;
use genome::Genome;
use super::{INFLOW_RATE_BASE, POND_HEIGHT, POND_WIDTH};

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
}