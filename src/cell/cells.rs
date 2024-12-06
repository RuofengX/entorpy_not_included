use std::collections::BTreeMap;

use kdtree::KdTree;

use crate::prelude::*;
use super::*;

// Cells
pub struct Cells {
    kd_idx: KdTree<Unit, CellID, Position>,
    pos_idx: BTreeMap<Position, CellID>,
    cells: Pool<CellID, Cell>,
}
impl Cells {
    pub async fn add(&mut self, value: Cell) -> CellID {
        let id = CellID::default();
        let pos = value.pos;
        self.cells.insert(id, value).await;
        self.kd_idx.add(pos, id).unwrap();
        self.pos_idx.insert(pos, id);
        id
    }

    pub async fn get_by_id(&self, id: CellID) -> Option<Pooling<Cell>> {
        self.cells.get(&id).await
    }

    pub async fn get_by_position(&self, pos: Position) -> Option<Pooling<Cell>> {
        let &id = self.pos_idx.get(&pos)?;
        self.get_by_id(id).await
    }

    // return None if there's no cell in self, otherwise always return Some(_)
    pub async fn get_nearest_by_position(&self, pos: Position) -> Option<Pooling<Cell>> {
        let (_, &id) = self
            .kd_idx
            .nearest(pos.as_ref(), 1, &kdtree::distance::squared_euclidean)
            .unwrap()[0];
        self.get_by_id(id).await
    }

    pub async fn get_nearest_by_id(&self, id: CellID) -> Option<Pooling<Cell>> {
        let pos = self.get_by_id(id).await?.as_ref().read().await.pos;
        self.get_nearest_by_position(pos).await
    }

    // search range

    pub fn iter_near_position(&self, pos: Position) -> AsyncIter {
        let iter: Vec<(Unit, &CellID)> = self
            .kd_idx
            .iter_nearest(pos.as_ref().as_ref(), &DEFAULT_DISTANCE)
            .unwrap()
            // .map(|(dis, &x)| (dis, x))
            .collect();
        AsyncIter { iter, cells: &self }
    }

    pub fn iter_within_range(&self, pos: Position, radius: Unit) -> AsyncIter {
        let iter: Vec<(Unit, &CellID)> = self
            .kd_idx
            .within(pos.as_ref(), radius, &DEFAULT_DISTANCE)
            .unwrap();
        AsyncIter { iter, cells: &self }
    }

    pub async fn get_neighbour_by_id(&self, id: CellID) -> Option<Vec<Pooling<Cell>>> {
        let c = self.get_by_id(id).await?;
        let p = c.read().await.pos;
        let mut ret = vec![];
        for n_pos in p.neighbour() {
            if let Some(n) = self.get_by_position(n_pos).await {
                ret.push(n);
            }
        }
        Some(ret)
    }
    pub async fn update(&self) {
        todo!("使用自身压力pressure和周边压力梯度gradients计算气体流向");
    }
}
