use core::cell::UnsafeCell;
use core::ptr::NonNull;
use core::sync::atomic::Ordering;
use crate::stable::util::{BorrowState, BorrowTy, NONZERO_1};
use crate::util::NonZeroExt;

#[derive(Debug)]
#[repr(C)]
struct LockState<T: ?Sized> {
    borrow: BorrowState,
    value: UnsafeCell<T>,
}

impl<T: ?Sized> LockState<T> {
    /*
    fn try_add_ref(&self) -> Option<()> {
        let cur = unsafe { *self.borrow.get(Ordering::Acquire) };
        match cur {
            BorrowState::None => {
                self.borrow.set(BorrowState::Ref(NONZERO_1));
                Some(())
            }
            BorrowState::Ref(val) => {
                self.borrow.set(BorrowState::Ref(val.checked_add(1)?));
                Some(())
            }
            _ => None,
        }
    }

    fn try_add_mut(&self) -> Option<()> {
        let cur = self.borrow.get();
        match cur {
            BorrowState::None => {
                self.borrow.set(BorrowState::Mut);
                Some(())
            }
            _ => None,
        }
    }

    /// Return a boolean indication whether this CellState should be dropped
    fn try_de_ref(&self) -> bool {
        let cur = self.borrow.get();
        match cur {
            BorrowState::Ref(val) => {
                match val.checked_sub(1) {
                    None => self.borrow.set(BorrowState::None),
                    Some(val) => self.borrow.set(BorrowState::Ref(val)),
                }
                false
            }
            BorrowState::Drop(BorrowTy::Ref(val)) => {
                if let Some(val) = val.checked_sub(1) {
                    self.borrow.set(BorrowState::Drop(BorrowTy::Ref(val)));
                    false
                } else {
                    true
                }
            }
            _ => false,
        }
    }

    /// Return a boolean indication whether this CellState should be dropped
    fn try_de_mut(&self) -> bool {
        let cur = self.borrow.get();
        match cur {
            BorrowState::Mut => {
                self.borrow.set(BorrowState::None);
                false
            }
            BorrowState::Drop(BorrowTy::Mut) => {
                true
            }
            _ => false,
        }
    }
     */

    unsafe fn val_ref<'a>(&self) -> &'a T {
        &*self.value.get()
    }

    unsafe fn val_mut<'a>(&self) -> &'a mut T {
        &mut *self.value.get()
    }
}

pub struct StableLock<T: ?Sized>(NonNull<LockState<T>>);
