use kdtree::KdTree;

mod cell;
mod player;
mod pos;

pub(crate) use cell::Cell;
pub(crate) use player::Player;
pub(crate) use pos::Position;

type Str = &'static str;

pub struct World {}

fn main() {
    let mut kd = KdTree::new(2);
    let cell = Cell::new(Position::default(), "void", 0.0);
    kd.add(cell.position(), cell).unwrap();

    println!("Hello, world!");
}
