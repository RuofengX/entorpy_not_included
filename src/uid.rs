use std::sync::atomic::AtomicU64;

use serde_derive::{Deserialize, Serialize};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// 单调递增、不一定连续
#[derive(Debug, Eq, Ord, Clone, Copy, Serialize, Deserialize)]
pub struct UID {
    pub id: u64,
}
impl PartialEq for UID {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}
impl PartialOrd for UID {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}
impl UID {
    pub fn new() -> Self {
        // todo!("未能正确初始化");
        let inner = COUNTER.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        Self { id: inner }
    }
    pub fn peek() -> u64 {
        COUNTER.load(std::sync::atomic::Ordering::Acquire)
    }
    pub fn set(value: u64) {
        COUNTER.store(value, std::sync::atomic::Ordering::Release);
    }
}
