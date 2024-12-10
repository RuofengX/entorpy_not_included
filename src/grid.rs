use ndarray::Array2;
use serde_derive::{Deserialize, Serialize};

use crate::{
    pooling::{pooling_new, Pool, Pooling},
    uid::UID,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    id: UID,
    data: CellData,
}
impl Cell {
    pub fn new(data: CellData) -> Self {
        let id = UID::new();
        Self { id, data }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CellData {
    Energy(i64),
    Blackhole(UID),
    Whitehole(UID),
}

#[derive(Debug)]
pub struct Grid<const N: usize> {
    data: Pool<UID, Cell>,
    index: Pooling<Array2<Option<UID>>>,
}
impl<const N: usize> Grid<N> {
    pub fn new() -> Self {
        let data = Pool::new();
        let index = pooling_new(Array2::from_shape_simple_fn((N, N), || None));
        Self { data, index }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PackedGrid<const N: usize> {
    data: Vec<(UID, Cell)>,
    index: Array2<Option<UID>>,
    max_id: u64,
}
impl<const N: usize> From<Grid<N>> for PackedGrid<N> {
    fn from(value: Grid<N>) -> Self {
        let data = value.data.to_vec();
        let index = value.index.blocking_read().clone();
        let max_id = UID::peek();
        Self {
            data,
            index,
            max_id,
        }
    }
}
impl<const N: usize> Into<Grid<N>> for PackedGrid<N> {
    fn into(self) -> Grid<N> {
        let data = self.data.into_iter().collect();
        let index = pooling_new(self.index);
        UID::set(self.max_id);
        Grid { data, index }
    }
}
