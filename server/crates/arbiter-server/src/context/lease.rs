use std::sync::Arc;

use dashmap::DashSet;

#[derive(Clone, Default)]
struct LeaseStorage<T: Eq + std::hash::Hash>(Arc<DashSet<T>>);

// A lease that automatically releases the item when dropped
pub struct Lease<T: Clone + std::hash::Hash + Eq> {
    item: T,
    storage: LeaseStorage<T>,
}
impl<T: Clone + std::hash::Hash + Eq> Drop for Lease<T> {
    fn drop(&mut self) {
        self.storage.0.remove(&self.item);
    }
}

#[derive(Clone, Default)]
pub struct LeaseHandler<T: Clone + std::hash::Hash + Eq> {
    storage: LeaseStorage<T>,
}

impl<T: Clone + std::hash::Hash + Eq> LeaseHandler<T> {
    pub fn new() -> Self {
        Self {
            storage: LeaseStorage(Arc::new(DashSet::new())),
        }
    }

    pub fn acquire(&self, item: T) -> Result<Lease<T>, ()> {
        if self.storage.0.insert(item.clone()) {
            Ok(Lease {
                item,
                storage: self.storage.clone(),
            })
        } else {
            Err(())
        }
    }
}
