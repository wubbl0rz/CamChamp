use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, RwLock, RwLockReadGuard};

fn lul()
{
    let r : Registry<u32> = Registry::new();
    r.add(2);

    for blub in r.all().iter() {

    }
}

pub struct Registry<T> {
    store: RwLock<HashMap<u32, Arc<T>>>
}

impl<T> Registry<T> {
    pub fn new() -> Self {
        Self {
            store: RwLock::new(HashMap::new())
        }
    }

    pub fn all(&self) -> RwLockReadGuard<HashMap<u32, Arc<T>>> {
        return self.store.read().unwrap();
    }

    pub fn add(&self, e : T) -> u32{
        let mut map = self.store.write().unwrap();

        let id = (0..u32::MAX)
            .into_iter()
            .find(|id| !map.contains_key(id))
            .expect("NO FREE ID FOUND");

        map.insert(id, Arc::new(e));
        return id;
    }

    pub fn get(&self, id : u32) -> Arc<T> {
        return self.store.read().unwrap().get(&id).unwrap().clone();
    }

    pub fn del(&self, id : u32) {
        self.store.write().unwrap().remove(&id);
    }
}
