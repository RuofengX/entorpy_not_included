use std::{collections::BTreeMap, sync::Arc};

use tokio::sync::RwLock;

pub type Pooling<T> = Arc<RwLock<T>>;
pub fn pooling_new<T>(value: T) -> Pooling<T> {
    Arc::new(RwLock::new(value))
}

#[derive(Debug, Clone)]
pub struct Pool<K, V> {
    inner: Pooling<BTreeMap<K, Pooling<V>>>,
}
impl<K, V> Pool<K, V> {
    pub fn new() -> Self {
        Pool {
            inner: Pooling::new(RwLock::new(BTreeMap::new())),
        }
    }
}

impl<K: Ord, V> Pool<K, V> {
    pub async fn insert(&self, key: K, value: V) -> Option<Pooling<V>> {
        let mut rtx = self.inner.write().await;
        rtx.insert(key, pooling_new(value)).map(|x| x)
    }
    pub fn blocking_insert(&self, key: K, value: V) -> Option<Pooling<V>> {
        let mut rtx = self.inner.blocking_write();
        rtx.insert(key, pooling_new(value)).map(|x| x)
    }
    pub async fn get(&self, key: &K) -> Option<Pooling<V>> {
        let rtx = self.inner.read().await;
        rtx.get(key).map(|x| Arc::clone(x))
    }
    pub fn blocking_get(&self, key: &K) -> Option<Pooling<V>> {
        let rtx = self.inner.blocking_read();
        rtx.get(key).map(|x| Arc::clone(x))
    }
    pub async fn take(&self, key: &K) -> Option<V> {
        todo!()
    }
    pub async fn remove(&self, key: &K) -> Option<Pooling<V>> {
        let mut rtx = self.inner.write().await;
        rtx.remove(key)
    }
    pub async fn blocking_remove(&self, key: &K) -> Option<Pooling<V>> {
        let mut rtx = self.inner.blocking_write();
        rtx.remove(key)
    }
}

impl<K: Copy, V: Clone> Pool<K, V> {
    pub fn to_vec(&self) -> Vec<(K, V)> {
        let inner = self.inner.blocking_read();
        let ret = inner
            .iter()
            .map(|(&k, v)| (k, v.blocking_read().clone()))
            .collect();
        ret
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for Pool<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let map: BTreeMap<K, Pooling<V>> =
            iter.into_iter().map(|(k, v)| (k, pooling_new(v))).collect();
        let inner = pooling_new(map);
        Self { inner }
    }
}
