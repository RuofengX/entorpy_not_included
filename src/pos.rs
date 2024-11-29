use ordered_float::OrderedFloat;
use serde_derive::{Deserialize, Serialize};

pub type Unit = OrderedFloat<f32>;
#[derive(
    Debug, Clone, Copy, Default, Hash, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Position([Unit; 2]);

impl AsRef<[Unit]> for Position {
    fn as_ref(&self) -> &[Unit] {
        &self.0
    }
}

impl Position {
    #[inline]
    pub fn up(&self) -> Position {
        let mut ret = self.clone();
        ret.0[1] += 1.0;
        ret
    }
    #[inline]
    pub fn down(&self) -> Position {
        let mut ret = self.clone();
        ret.0[1] -= 1.0;
        ret
    }
    #[inline]
    pub fn left(&self) -> Position {
        let mut ret = self.clone();
        ret.0[0] -= 1.0;
        ret
    }
    #[inline]
    pub fn right(&self) -> Position {
        let mut ret = self.clone();
        ret.0[0] += 1.0;
        ret
    }
    #[inline]
    pub fn around(&self) -> [Position; 8] {
        [
            self.up(),
            self.down(),
            self.left(),
            self.right(),
            self.up().left(),
            self.up().right(),
            self.down().left(),
            self.down().right(),
        ]
    }
}
