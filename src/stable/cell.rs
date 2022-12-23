//! A non-thread-safe stable cell

use alloc::boxed::Box;
use core::cell::{Cell, UnsafeCell};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
#[cfg(feature = "unstable")]
use core::marker::Unsize;
#[cfg(feature = "unstable")]
use core::ops::CoerceUnsized;
use crate::stable::util::BorrowState;

#[derive(Debug)]
#[repr(C)]
struct CellState<T: ?Sized> {
    borrow: Cell<BorrowState>,
    value: UnsafeCell<T>,
}

impl<T: ?Sized> CellState<T> {
    fn try_add_ref(&self) -> Option<()> {
        let cur = self.borrow.get();
        match cur.incr_ref() {
            Some(new) => {
                self.borrow.set(new);
                Some(())
            }
            None => None,
        }
    }

    fn try_add_mut(&self) -> Option<()> {
        let cur = self.borrow.get();
        match cur.incr_mut() {
            Some(new) => {
                self.borrow.set(new);
                Some(())
            }
            _ => None,
        }
    }

    /// Return a boolean indication whether this `CellState` should be dropped
    fn try_de_ref(&self) -> bool {
        let cur = self.borrow.get();
        let (new, drop) = cur.decr_ref();
        self.borrow.set(new);
        drop
    }

    /// Return a boolean indication whether this `CellState` should be dropped
    fn try_de_mut(&self) -> bool {
        let cur = self.borrow.get();
        let (new, drop) = cur.decr_mut();
        self.borrow.set(new);
        drop
    }

    unsafe fn val_ref<'a>(&self) -> &'a T {
        &*self.value.get()
    }

    unsafe fn val_mut<'a>(&self) -> &'a mut T {
        &mut *self.value.get()
    }
}

impl<T> CellState<T> {
    fn new(val: T) -> CellState<T> {
        CellState {
            borrow: Cell::new(BorrowState::new()),
            value: UnsafeCell::new(val),
        }
    }
}

#[cfg(feature = "unstable")]
impl<T: CoerceUnsized<U>, U> CoerceUnsized<CellState<U>> for CellState<T> {}

/// A stable cell. Mixes traits of an `Rc` and a `RefCell` - Borrows may outlive the cell itself,
/// and also may be mutable due to the single-ownership.
pub struct StableCell<T: ?Sized>(NonNull<CellState<T>>);

impl<T: ?Sized> StableCell<T> {
    /// Create a new `StableCell` from a type which unsizes to the cell type
    #[cfg(feature = "unstable")]
    pub fn new_from<U: Unsize<T>>(val: U) -> StableCell<T> {
        let ptr = Box::leak(Box::new(CellState::new(val)) as Box<CellState<T>>);
        StableCell(NonNull::from(ptr))
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

impl<T> StableCell<T> {
    pub fn new(val: T) -> StableCell<T> {
        let ptr = Box::leak(Box::new(CellState::new(val)));
        StableCell(NonNull::from(ptr))
    }
}

impl<T> Clone for StableCell<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        StableCell::new(self.try_borrow().expect("Couldn't borrow value to clone").clone())
    }
}

impl<T: ?Sized> Drop for StableCell<T> {
    fn drop(&mut self) {
        let state = unsafe { self.0.as_ref() };
        let borrow = state.borrow.get();
        if borrow.is_none() {
            unsafe { Box::from_raw(self.0.as_ptr()) };
        } else {
            state.borrow.set(borrow.make_drop());
        }
    }
}

#[derive(Debug)]
pub struct StableRef<'a, T: ?Sized> {
    state: NonNull<CellState<T>>,
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
    state: NonNull<CellState<T>>,
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
        let cell = StableCell::<[i32]>::new_from([1, 2, 3]);
        let b = cell.try_borrow().unwrap();
        assert_eq!(&*b, &[1, 2, 3]);
    }

    #[test]
    fn test_borrow() {
        let cell = StableCell::new(5);
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
        let cell = StableCell::new(5);

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
        let cell = StableCell::new(-1);
        let b = cell.try_borrow().unwrap();
        let _b2 = cell.try_borrow().unwrap();
        drop(cell);
        assert_eq!(*b, -1);
    }

    #[test]
    fn test_drop_borrow_mut() {
        let cell = StableCell::new(-1);
        let b = cell.try_borrow_mut().unwrap();
        drop(cell);
        assert_eq!(*b, -1);
    }
}
