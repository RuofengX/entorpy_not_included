mod cells;
mod material;

use std::{
    async_iter::AsyncIterator,
    future::Future,
    ops::Deref,
    pin::{pin, Pin},
    sync::atomic::AtomicUsize,
    task::{Context, Poll},
};

use cells::Cells;
use futures::future::join_all;
use serde_derive::{Deserialize, Serialize};

use crate::{pos::Offset, prelude::*};

pub use material::MaterialTy;

const DEFAULT_DISTANCE: for<'a, 'b> fn(&'a [Unit], &'b [Unit]) -> Unit =
    kdtree::distance::squared_euclidean;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
pub struct Material {
    ty: MaterialTy,
    mass: f32,
    temperature: f32,
    /// 只有气体才有压力
    pressure: Option<f32>,
}
impl Material {
    /// 体积
    pub const VOLUMN: f32 = 1.0;

    /// 新建一个Cell，认为type_hint是有效的字符串
    pub fn new_unchecked(type_hint: &str, mass: f32, temperature: f32) -> Self {
        let mut ret = Material {
            ty: MaterialTy::get_unchecked(&type_hint),
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
            self.ty = MaterialTy::get_unchecked("void");
            self.mass = f32::NAN;
            self.temperature = f32::NAN;
            self.pressure = None;
            return;
        }

        if let Some(new_ty) = self.ty.check_transition(self.temperature) {
            self.ty = new_ty;
        }

        self.pressure = self
            .ty
            .gas_pressure(self.mass, self.temperature, Self::VOLUMN);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub id: CellID,
    pub pos: Position,
    material: Option<Material>, // None if cell is void
}
impl Cell {
    pub fn get_material(&self) -> Option<&Material> {
        self.material.as_ref()
    }

    pub fn get_gas_pressure(&self) -> Option<f32> {
        self.get_material().and_then(|x| x.pressure)
    }

    pub fn is_gas(&self) -> bool {
        self.get_material().is_some_and(|x| x.pressure.is_some())
    }

    pub fn is_void(&self) -> bool {
        self.get_material().is_none()
    }

    pub async fn gas_force(&self, cells: &Cells) -> Option<Offset> {
        if !self.is_gas() {
            return None;
        }

        let task = self.pos.neighbour_with_offset().into_iter();
        let gradients = join_all(task.map(|(pos, offset)| async move {
            if let Some(cell) = cells.get_by_position(pos).await {
                Some((cell.read().await.get_gas_pressure()?, offset))
            } else {
                None
            }
        }))
        .await;

        let gradient: Offset = gradients
            .into_iter()
            .filter_map(|x| x)
            .map(|(pressure, offset)| offset * pressure)
            .sum();

        Some(gradient)
    }
}
impl AsRef<Position> for Cell {
    fn as_ref(&self) -> &Position {
        &self.pos
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
