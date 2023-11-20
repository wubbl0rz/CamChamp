use std::collections::hash_map::RandomState;
use std::sync::Arc;

use dashmap::DashMap;
use dashmap::iter::Iter;

pub struct Registry<T> {
    store: DashMap<u32, Arc<T>>
}

impl<T> Registry<T> {
    pub fn new() -> Self {
        Self {
            store: DashMap::new()
        }
    }

    pub fn len(&self) -> usize {
        return self.store.len();
    }

    pub fn iter(&self) -> Iter<u32, Arc<T>, RandomState, DashMap<u32, Arc<T>>> {
        return self.store.iter();
    }

    pub fn add<F>(&self, cb: F) -> (u32, Arc<T>) where F: FnOnce(u32) -> T
    {
        let id = (0..u32::MAX)
            .into_iter()
            .find(|id| !self.store.contains_key(id))
            .expect("NO FREE ID FOUND");

        let t = cb(id);
        let arc = Arc::new(t);
        self.store.insert(id, arc.clone());

        return (id, arc.clone());
    }

    pub fn get(&self, id : u32) -> Option<Arc<T>> {
        return match self.store.get(&id) {
            None => None,
            Some(r) => Some(r.value().clone())
        };
    }

    pub fn del(&self, id : u32) {
        self.store.remove(&id);
    }
}
