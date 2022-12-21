use alloc::boxed::Box;
use core::cell::{Cell, UnsafeCell};
use core::{mem, ptr};
use core::marker::PhantomData;
use core::num::NonZeroU64;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use alloc::alloc::Layout;
use crate::util::{NonZeroExt, PtrRepr};

const NONZERO_1: NonZeroU64 = unsafe { NonZeroU64::new_unchecked(1) };

#[derive(Debug, Copy, Clone)]
enum DropState {
    Mut,
    Ref(NonZeroU64),
}

const _: () = assert!(mem::size_of::<DropState>() == mem::size_of::<u64>());

#[derive(Debug, Copy, Clone)]
enum BorrowState {
    None,
    Ref(NonZeroU64),
    Mut,
    Drop(DropState),
}

enum BorrowChange {
    AddRef,
    AddMut,
    DeRef,
    DeMut,
}

#[derive(Debug)]
#[repr(C)]
struct CellState<T: ?Sized> {
    borrow: Cell<BorrowState>,
    value: UnsafeCell<T>,
}

impl<T: ?Sized> CellState<T> {
    fn try_borrow(&self, change: BorrowChange) -> Option<BorrowState> {
        let cur = self.borrow.get();
        match (cur, change) {
            (BorrowState::None, BorrowChange::AddRef) => {
                self.borrow.set(BorrowState::Ref(NONZERO_1));
                Some(cur)
            }
            (BorrowState::None, BorrowChange::AddMut) => {
                self.borrow.set(BorrowState::Mut);
                Some(cur)
            }
            (BorrowState::None, _) => None,

            (BorrowState::Ref(val), BorrowChange::AddRef) => {
                self.borrow.set(BorrowState::Ref(val.checked_add(1)?));
                Some(cur)
            }
            (BorrowState::Ref(val), BorrowChange::DeRef) => {
                match val.checked_sub(1) {
                    None => self.borrow.set(BorrowState::None),
                    Some(val) => self.borrow.set(BorrowState::Ref(val)),
                }
                Some(cur)
            }
            (BorrowState::Ref(_), _) => None,

            (BorrowState::Mut, BorrowChange::DeMut) => {
                self.borrow.set(BorrowState::None);
                Some(cur)
            }
            (BorrowState::Mut, _) => None,

            (BorrowState::Drop(DropState::Ref(val)), BorrowChange::DeRef) => {
                if let Some(val) = val.checked_sub(1) {
                    self.borrow.set(BorrowState::Drop(DropState::Ref(val)));
                }
                Some(cur)
            }
            (BorrowState::Drop(DropState::Mut), BorrowChange::DeMut) => {
                Some(cur)
            }
            (BorrowState::Drop(_), _) => None,
        }
    }

    unsafe fn val_ref<'a>(&self) -> &'a T {
        &*self.value.get()
    }

    unsafe fn val_mut<'a>(&self) -> &'a mut T {
        &mut *self.value.get()
    }
}

impl<T: ?Sized> CellState<T> {
    fn alloc(val: &T) -> NonNull<CellState<T>> {
        let layout = Layout::new::<Cell<BorrowState>>()
            .extend(Layout::for_value(val))
            .unwrap()
            .0;

        let meta = PtrRepr::new(val as *const T as *mut T)
            .metadata();

        let raw_ptr = unsafe { alloc::alloc::alloc(layout) }.cast();

        let raw_ptr = PtrRepr::from_meta_ptr(meta, raw_ptr).ptr();

        unsafe { NonNull::new_unchecked(raw_ptr) }
    }

    fn new_boxed(val: Box<T>) -> NonNull<CellState<T>> {
        let alloc = Self::alloc(&val);
        unsafe { (*alloc.as_ptr()).borrow = Cell::new(BorrowState::None) };
        let size = mem::size_of_val(&*val);
        unsafe { ptr::copy(
            (&*val as *const T).cast::<u8>(),
            ptr::addr_of_mut!((*alloc.as_ptr()).value).cast::<u8>(),
            size,
        ) };
        alloc
    }
}

impl<T> CellState<T> {
    fn new(val: T) -> CellState<T> {
        CellState {
            borrow: Cell::new(BorrowState::None),
            value: UnsafeCell::new(val),
        }
    }
}

pub struct StableCell<T: ?Sized>(NonNull<CellState<T>>);

impl<T: ?Sized> StableCell<T> {
    pub fn new_boxed(val: Box<T>) -> StableCell<T> {
        StableCell(CellState::new_boxed(val))
    }

    pub fn try_borrow<'a>(&self) -> Option<StableRef<'a, T>> {
        let state = unsafe { self.0.as_ref() };
        state.try_borrow(BorrowChange::AddRef)
            .map(|_| StableRef { state: self.0, _phantom: PhantomData })
    }

    pub fn try_borrow_mut<'a>(&self) -> Option<StableMut<'a, T>> {
        let state = unsafe { self.0.as_ref() };
        state.try_borrow(BorrowChange::AddMut)
            .map(|_| StableMut { state: self.0, _phantom: PhantomData })
    }
}

impl<T> StableCell<T> {
    pub fn new(val: T) -> StableCell<T> {
        let ptr = Box::leak(Box::new(CellState::new(val)));
        StableCell(NonNull::from(ptr))
    }
}

impl<T: ?Sized> Drop for StableCell<T> {
    fn drop(&mut self) {
        let state = unsafe { self.0.as_ref() };
        match state.borrow.get() {
            BorrowState::Ref(val) => {
                state.borrow.set(BorrowState::Drop(DropState::Ref(val)))
            }
            BorrowState::Mut => {
                state.borrow.set(BorrowState::Drop(DropState::Mut))
            }
            _ => {
                unsafe { Box::from_raw(self.0.as_ptr()) };
            }
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
        match state.try_borrow(BorrowChange::DeRef) {
            Some(BorrowState::Drop(DropState::Ref(NONZERO_1))) => {
                unsafe { Box::from_raw(self.state.as_ptr()) };
            }
            _ => (),
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
        match state.try_borrow(BorrowChange::DeMut) {
            Some(BorrowState::Drop(DropState::Mut)) => {
                unsafe { Box::from_raw(self.state.as_ptr()) };
            }
            _ => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
