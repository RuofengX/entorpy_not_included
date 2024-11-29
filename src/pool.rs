use std::{collections::BTreeMap, sync::Arc};

use tokio::sync::RwLock;

pub type Pooling<T> = Arc<RwLock<T>>;

#[derive(Debug, Clone)]
pub struct Pool<K, V> {
    inner: Pooling<BTreeMap<K, Pooling<V>>>,
}

impl<K: Ord, V> Pool<K, V> {
    pub async fn insert(&self, key: K, value: V) -> Option<Pooling<V>> {
        let mut rtx = self.inner.write().await;
        rtx.insert(key, Arc::new(RwLock::new(value))).map(|x| x)
    }
    pub async fn get(&self, key: &K) -> Option<Pooling<V>> {
        let rtx = self.inner.read().await;
        rtx.get(key).map(|x| Arc::clone(x))
    }
    pub async fn remove(&self, key: &K) -> Option<Pooling<V>> {
        let mut rtx = self.inner.write().await;
        rtx.remove(key)
    }
}
