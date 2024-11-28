use std::{
    ops::Deref,
    sync::{atomic::AtomicU64, Arc},
};

use serde_derive::{Deserialize, Serialize};

use crate::pos::Position;

#[derive(Debug, Clone, Default)]
pub struct Player {
    id: PlayerID,
    data: Arc<PlayerData>,
}
impl std::hash::Hash for Player {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Debug, Clone, Copy, Hash)]
struct PlayerID(u64);
impl PlayerID {
    const GLOBAL_COUNTER: AtomicU64 = AtomicU64::new(0);
}
impl Default for PlayerID {
    fn default() -> Self {
        let id = Self::GLOBAL_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Self(id)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct PlayerData {
    pos: Position,
    health: f32,
    speed: f32,
    energy: f32,
    breath: f32,
}
impl Default for PlayerData {
    fn default() -> Self {
        Self {
            pos: Default::default(),
            health: 100.0,
            speed: 1.0,
            energy: 4000.0,
            breath: 100.0,
        }
    }
}