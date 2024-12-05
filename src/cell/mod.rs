mod material;

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

use crate::prelude::*;

pub use material::Material;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pos: Position,
    material: Material,
    mass: f32,
    temperature: f32,
    /// 只有气体才有压力
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

    /// 新建一个Cell，认为type_hint是有效的字符串
    pub fn new_unchecked(pos: Position, type_hint: &str, mass: f32, temperature: f32) -> Self {
        let mut ret = Cell {
            pos,
            material: Material::get_unchecked(&type_hint),
            mass,
            temperature,
            pressure: None,
        };
        ret.update();
        ret
    }

    /// 向cell堆叠新的物质，mass必须为非负数
    pub fn stack(&mut self, mass: f32) {
        if mass < 0.0 {
            return;
        }
        self.mass += mass;
        self.update();
    }

    /// 从cell拿取其中的物质，拿取后的质量必须为非负数
    pub fn take(&mut self, mass: f32) -> Option<f32> {
        let mass = self.mass - mass;
        if mass >= 0.0 {
            self.mass = mass;
            self.update();
            Some(mass)
        } else {
            None
        }
    }

    /// 对cell加热或排热
    pub fn heat(&mut self, temperature: f32) {
        self.temperature += temperature;
        self.update();
    }

    /// 更新Cell数据状态
    pub fn update(&mut self) {
        if self.mass == 0.0 {
            self.material = Material::get_unchecked("void");
            self.mass = f32::NAN;
            self.temperature = f32::NAN;
            self.pressure = None;
            return;
        }

        if let Some(new_ty) = self.material.check_transition(self.temperature) {
            self.material = new_ty;
        }

        self.pressure = self
            .material
            .gas_pressure(self.mass, self.temperature, Self::VOLUMN);
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
