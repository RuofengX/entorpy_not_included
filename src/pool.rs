use std::{collections::BTreeMap, sync::Arc};

use serde::de::Deserializer;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
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
    pub fn blocking_get(&self, key: &K) -> Option<Pooling<V>> {
        let rtx = self.inner.blocking_read();
        rtx.get(key).map(|x| Arc::clone(x))
    }
    pub async fn remove(&self, key: &K) -> Option<Pooling<V>> {
        let mut rtx = self.inner.write().await;
        rtx.remove(key)
    }
}

pub mod pooling_serde {
    use super::*;

    pub fn serialize<S, T>(val: &Pooling<T>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        T::serialize(&*val.blocking_read(), s)
    }

    pub fn deserialize<'de, D, T>(d: D) -> Result<Arc<RwLock<T>>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        Ok(Arc::new(RwLock::new(T::deserialize(d)?)))
    }
}
