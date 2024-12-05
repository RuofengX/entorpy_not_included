#![feature(async_iterator)]

pub mod cell;
pub mod player;
pub mod pool;
pub mod pos;
pub mod res;

pub mod prelude {
    pub use crate::cell::Cell;
    pub use crate::player::Player;
    pub use crate::pool::{Pool, Pooling};
    pub use crate::pos::{Position, Unit};
}

pub struct World {}

fn main() {
    println!("Hello, world!");
}
