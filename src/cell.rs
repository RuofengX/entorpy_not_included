use std::{
    async_iter::AsyncIterator,
    collections::BTreeMap,
    future::Future,
    ops::Deref,
    pin::{pin, Pin},
    sync::atomic::AtomicUsize,
    task::{Context, Poll},
};

use kdtree::KdTree;
use serde_derive::{Deserialize, Serialize};

use crate::{prelude::*, res::CellTy};

const DEFAULT_DISTANCE: for<'a, 'b> fn(&'a [Unit], &'b [Unit]) -> Unit =
    kdtree::distance::squared_euclidean;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CellID(usize);
impl CellID {
    const GLOBAL_COUNTER: AtomicUsize = AtomicUsize::new(0);
}
impl Default for CellID {
    fn default() -> Self {
        let id = Self::GLOBAL_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Self::new(id)
    }
}
impl AsRef<usize> for CellID {
    fn as_ref(&self) -> &usize {
        &self.0
    }
}
impl CellID {
    pub fn new(value: usize) -> Self {
        Self(value)
    }
}
impl From<usize> for CellID {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}
impl Deref for CellID {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Cell {
    pos: Position,
    type_hint: &'static str,
    mass: f32,
    temperature: f32,
    /// only gas has pressure
    pressure: Option<f32>,
}
impl AsRef<Position> for Cell {
    fn as_ref(&self) -> &Position {
        &self.pos
    }
}
impl Cell {
    /// 体积
    pub const VOLUMN: f32 = 1.0;

    /// No check type_hint
    pub fn new_unchecked(
        pos: Position,
        type_hint: &'static str,
        mass: f32,
        temperature: f32,
    ) -> Self {
        let ty = CellTy::get_unchecked(&type_hint);
        let pressure = ty.gas_pressure(mass, temperature, Self::VOLUMN);
        Cell {
            pos,
            type_hint,
            mass,
            temperature,
            pressure,
        }
    }

    #[inline]
    pub fn ty(&self) -> &'static CellTy {
        CellTy::get_unchecked(self.type_hint)
    }

    pub fn tick(&mut self) {
        self.pressure = self
            .ty()
            .gas_pressure(self.mass, self.temperature, Self::VOLUMN);
        if let Some(new_ty) = self.ty().check_transition(self.temperature) {
            self.type_hint = new_ty.name.as_str();
        }
    }
}

// Cells
pub struct Cells {
    kd_idx: KdTree<Unit, CellID, Position>,
    pos_idx: BTreeMap<Position, CellID>,
    cells: Pool<CellID, Cell>,
}
impl Cells {
    pub async fn add(&mut self, value: Cell) -> CellID {
        let id = CellID::default();
        self.cells.insert(id, value).await;
        self.kd_idx.add(value.pos, id).unwrap();
        self.pos_idx.insert(value.pos, id);
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
}

pub struct AsyncIter<'s> {
    pub(crate) iter: Vec<(Unit, &'s CellID)>,
    pub(crate) cells: &'s Cells,
}
impl AsyncIterator for AsyncIter<'_> {
    type Item = (Unit, Pooling<Cell>);

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some((dis, id)) = self.iter.pop() {
            let future = pin!(self.cells.get_by_id(*id));
            match future.poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(cell) => {
                    let ret = cell.map(|x| (dis, x));
                    Poll::Ready(ret)
                }
            }
        } else {
            Poll::Ready(None)
        }
    }
}
