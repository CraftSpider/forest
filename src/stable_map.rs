use std::cell::{Cell, RefCell};
use std::ptr::NonNull;
use slotmap::{Key, SecondaryMap, SlotMap};
use typed_arena::Arena;

#[derive(Debug, Copy, Clone)]
pub enum BorrowState {
    None = 0,
    Ref,
    Mut,
}

/*#[derive(Copy, Clone)]
pub struct StableKey<K: Key = DefaultKey>(K);*/

/// A variation on a SlotMap with references to contained items being move-safe. All contained
/// items are boxed and refcell-tracked.
pub struct StableMap<K: Key, T> {
    arena: Arena<RefCell<T>>,
    inner: SlotMap<K, NonNull<T>>,
    borrowed: SecondaryMap<K, Cell<BorrowState>>,
}

impl<K: Key, T> StableMap<K, T> {
    pub fn with_key() -> StableMap<K, T> {
        StableMap {
            arena: Arena::new(),
            inner: SlotMap::with_key(),
            borrowed: SecondaryMap::new(),
        }
    }

    pub fn get(&self, key: K) -> Option<!> {
        todo!()
    }

    pub fn insert(&mut self, item: T) -> K {
        let key = self.inner.insert(NonNull::from(Box::leak(Box::new(item))));
        self.borrowed.insert(key, Cell::new(BorrowState::None));
        key
    }

    pub fn remove(&mut self, key: K) -> Option<T> {
        let key = self.inner.remove(key);
        key.map(|ptr| *unsafe { Box::from_raw(ptr.as_ptr()) })
    }
}
