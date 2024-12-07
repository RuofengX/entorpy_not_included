#![feature(async_iterator)]
#![feature(async_closure)]
#![feature(portable_simd)]

pub mod cell;
pub mod player;
pub mod pool;
pub mod pos;
mod pyhsic;

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
