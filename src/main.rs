#![feature(async_iterator)]

use grid::Grid;
use uid::UID;

pub mod grid;
pub mod player;
pub mod pooling;
pub mod uid;

fn main() {
    println!("Hello, world!");
    let g: Grid<4> = grid::Grid::new();
    println!("{:?}", g);


    println!("{:?}", UID::new());
    println!("{:?}", UID::new());
    println!("{:?}", UID::new());
    println!("{:?}", UID::new());
}
