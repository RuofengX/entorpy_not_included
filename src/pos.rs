use ordered_float::OrderedFloat;
use serde_derive::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, Default, Hash, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Position([OrderedFloat<f32>; 2]);

impl AsRef<[OrderedFloat<f32>]> for Position {
    fn as_ref(&self) -> &[OrderedFloat<f32>] {
        &self.0
    }
}

impl Position {
    pub fn up(&self) -> Position {
        let mut ret = self.clone();
        ret.0[1] + 1;
    }
}
