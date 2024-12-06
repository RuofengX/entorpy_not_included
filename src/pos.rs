use std::ops::Add;

use glam::Vec2;
use ordered_float::OrderedFloat;
use serde_derive::{Deserialize, Serialize};

pub type Unit = OrderedFloat<f32>;
pub type Offset = Vec2;

pub const ORIGIN: Offset = Offset { x: 0.0, y: 0.0 };
pub const EMPTY: Offset = ORIGIN;
pub const UP: Offset = Offset { x: 0.0, y: -1.0 };
pub const DOWN: Offset = Offset { x: 0.0, y: 1.0 };
pub const LEFT: Offset = Offset { x: -1.0, y: 0.0 };
pub const RIGHT: Offset = Offset { x: 1.0, y: 0.0 };
pub const LEFT_UP: Offset = Offset { x: -1.0, y: -1.0 };
pub const RIGHT_UP: Offset = Offset { x: 1.0, y: -1.0 };
pub const LEFT_DOWN: Offset = Offset { x: -1.0, y: 1.0 };
pub const RIGHT_DOWN: Offset = Offset { x: 1.0, y: 1.0 };

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
    pub fn neighbour(self) -> [Position; 8] {
        [
            self + UP,
            self + DOWN,
            self + LEFT,
            self + RIGHT,
            self + LEFT_UP,
            self + RIGHT_UP,
            self + LEFT_DOWN,
            self + RIGHT_DOWN,
        ]
    }

    pub fn neighbour_with_offset(self) -> [(Position, Offset); 8] {
        [
            (self + UP, UP),
            (self + DOWN, DOWN),
            (self + LEFT, LEFT),
            (self + RIGHT, RIGHT),
            (self + LEFT_UP, LEFT_UP),
            (self + RIGHT_UP, RIGHT_UP),
            (self + LEFT_DOWN, LEFT_DOWN),
            (self + RIGHT_DOWN, RIGHT_DOWN),
        ]
    }

    #[inline]
    pub fn offset_from_zero(&self) -> Offset {
        Offset::from_array(self.0.map(|x| x.0))
    }
}

impl Into<Offset> for Position {
    fn into(self) -> Offset {
        Offset::from_array(self.0.map(|x| x.0))
    }
}

impl From<Offset> for Position {
    fn from(value: Offset) -> Self {
        Self(value.to_array().map(|x| OrderedFloat::<f32>::from(x)))
    }
}

impl Add<Offset> for Position {
    type Output = Position;

    fn add(self, rhs: Offset) -> Self::Output {
        let ret = self.offset_from_zero() + rhs;
        ret.into()
    }
}
