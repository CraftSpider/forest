//! A thread-safe stable cell

use core::cell::UnsafeCell;
use core::ptr::NonNull;
use core::marker::PhantomData;
#[cfg(feature = "unstable")]
use core::marker::Unsize;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicUsize, Ordering};
use alloc::boxed::Box;
use crate::stable::util::BorrowState;

#[derive(Debug)]
#[repr(C)]
struct LockState<T: ?Sized> {
    borrow: AtomicUsize,
    value: UnsafeCell<T>,
}

impl<T: ?Sized> LockState<T> {
    fn try_add_ref(&self) -> Option<()> {
        self.borrow.fetch_update(
            Ordering::AcqRel,
            Ordering::Acquire,
            |cur| {
                BorrowState::from_val(cur).incr_ref().map(BorrowState::to_val)
            })
            .map(|_| ())
            .ok()
    }

    fn try_add_mut(&self) -> Option<()> {
        self.borrow.fetch_update(
            Ordering::AcqRel,
            Ordering::Acquire,
            |cur| {
                BorrowState::from_val(cur).incr_mut().map(BorrowState::to_val)
            })
            .map(|_| ())
            .ok()
    }

    /// Return a boolean indication whether this `LockState` should be dropped
    fn try_de_ref(&self) -> bool {
        let mut drop_flag = false;
        let _ = self.borrow.fetch_update(
            Ordering::AcqRel,
            Ordering::Acquire,
            |cur| {
                let (out, drop) = BorrowState::from_val(cur).decr_ref();
                drop_flag = drop;
                Some(out.to_val())
            });
        drop_flag
    }

    /// Return a boolean indication whether this `LockState` should be dropped
    fn try_de_mut(&self) -> bool {
        let mut drop_flag = false;
        let _ = self.borrow.fetch_update(
            Ordering::AcqRel,
            Ordering::Acquire,
            |cur| {
                let (out, drop) = BorrowState::from_val(cur).decr_mut();
                drop_flag = drop;
                Some(out.to_val())
            });
        drop_flag
    }

    unsafe fn val_ref<'a>(&self) -> &'a T {
        &*self.value.get()
    }

    unsafe fn val_mut<'a>(&self) -> &'a mut T {
        &mut *self.value.get()
    }
}

impl<T> LockState<T> {
    fn new(val: T) -> LockState<T> {
        LockState {
            borrow: AtomicUsize::new(BorrowState::new().to_val()),
            value: UnsafeCell::new(val),
        }
    }
}

pub struct StableLock<T: ?Sized>(NonNull<LockState<T>>);

impl<T: ?Sized> StableLock<T> {
    /// Create a new `StableLock` from a type which unsizes to the cell type
    #[cfg(feature = "unstable")]
    pub fn new_from<U: Unsize<T>>(val: U) -> StableLock<T> {
        let ptr = Box::leak(Box::new(LockState::new(val)) as Box<LockState<T>>);
        StableLock(NonNull::from(ptr))
    }

    /// Attempt to get a shared borrow to this cell. The borrow may live as long as `T`
    pub fn try_borrow<'a>(&self) -> Option<StableRef<'a, T>> {
        let state = unsafe { self.0.as_ref() };
        state.try_add_ref()
            .map(|_| StableRef { state: self.0, _phantom: PhantomData })
    }

    /// Attempt to get a unique borrow to this cell. The borrow may live as long as `T`
    pub fn try_borrow_mut<'a>(&self) -> Option<StableMut<'a, T>> {
        let state = unsafe { self.0.as_ref() };
        state.try_add_mut()
            .map(|_| StableMut { state: self.0, _phantom: PhantomData })
    }
}

impl<T> StableLock<T> {
    pub fn new(val: T) -> StableLock<T> {
        let ptr = Box::leak(Box::new(LockState::new(val)));
        StableLock(NonNull::from(ptr))
    }
}

unsafe impl<T: ?Sized + Send> Send for StableLock<T> {}
unsafe impl<T: ?Sized + Send> Sync for StableLock<T> {}

impl<T: ?Sized> Drop for StableLock<T> {
    fn drop(&mut self) {
        let mut drop_flag = false;
        let state = unsafe { self.0.as_ref() };
        let _ = state.borrow.fetch_update(
            Ordering::AcqRel,
            Ordering::Acquire,
            |cur| {
                let state = BorrowState::from_val(cur);
                if state.is_none() {
                    drop_flag = true;
                    None
                } else {
                    Some(state.make_drop().to_val())
                }
            });
        if drop_flag {
            unsafe { Box::from_raw(self.0.as_ptr()) };
        }
    }
}

#[derive(Debug)]
pub struct StableRef<'a, T: ?Sized> {
    state: NonNull<LockState<T>>,
    _phantom: PhantomData<&'a T>,
}

impl<T: ?Sized> Deref for StableRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.state.as_ref().val_ref() }
    }
}

impl<T: ?Sized + PartialEq> PartialEq for StableRef<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<T: ?Sized> Drop for StableRef<'_, T> {
    fn drop(&mut self) {
        let state = unsafe { self.state.as_ref() };
        if state.try_de_ref() {
            unsafe { Box::from_raw(self.state.as_ptr()) };
        }
    }
}

#[derive(Debug)]
pub struct StableMut<'a, T: ?Sized> {
    state: NonNull<LockState<T>>,
    _phantom: PhantomData<&'a mut T>,
}

impl<T: ?Sized + PartialEq> PartialEq for StableMut<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<T: ?Sized> Deref for StableMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.state.as_ref().val_ref() }
    }
}

impl<T: ?Sized> DerefMut for StableMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.state.as_ref().val_mut() }
    }
}

impl<T: ?Sized> Drop for StableMut<'_, T> {
    fn drop(&mut self) {
        let state = unsafe { self.state.as_ref() };
        if state.try_de_mut() {
            unsafe { Box::from_raw(self.state.as_ptr()) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "unstable")]
    fn test_unsized() {
        let cell = StableLock::<[i32]>::new_from([1, 2, 3]);
        let b = cell.try_borrow().unwrap();
        assert_eq!(&*b, &[1, 2, 3]);
    }

    #[test]
    fn test_borrow() {
        let cell = StableLock::new(5);
        assert_eq!(
            cell.try_borrow().as_deref(),
            Some(&5),
        );

        let b1 = cell.try_borrow().unwrap();
        let b2 = cell.try_borrow().unwrap();
        assert_eq!(b1, b2);

        assert_eq!(cell.try_borrow_mut(), None);

        drop(b1);
        drop(b2);
    }

    #[test]
    fn test_borrow_mut() {
        let cell = StableLock::new(5);

        assert_eq!(
            cell.try_borrow_mut().as_deref_mut(),
            Some(&mut 5),
        );

        let b1 = cell.try_borrow_mut().unwrap();

        assert_eq!(
            cell.try_borrow_mut(),
            None,
        );

        drop(b1);
    }

    #[test]
    fn test_drop_borrow() {
        let cell = StableLock::new(-1);
        let b = cell.try_borrow().unwrap();
        let _b2 = cell.try_borrow().unwrap();
        drop(cell);
        assert_eq!(*b, -1);
    }

    #[test]
    fn test_drop_borrow_mut() {
        let cell = StableLock::new(-1);
        let b = cell.try_borrow_mut().unwrap();
        drop(cell);
        assert_eq!(*b, -1);
    }
}
