use serde_derive::{Deserialize, Serialize};

use crate::Position;

type CellType = &'static str;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Cell {
    pos: Position,
    ty: &'static str,
    amount: f32,
}
impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}
impl AsRef<Position> for Cell {
    fn as_ref(&self) -> &Position {
        &self.pos
    }
}
impl Cell {
    pub fn new(pos: Position, ty: &'static str, amount: f32) -> Self {
        Cell { pos, ty, amount }
    }

    #[inline]
    pub fn position(&self) -> Position {
        self.pos
    }

    #[inline]
    pub fn data(&self) -> (CellType, f32) {
        (self.ty, self.amount)
    }
}
