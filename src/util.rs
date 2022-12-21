use core::num::NonZeroU64;
use core::mem;

pub trait NonZeroExt: Sized {
    type Inner;

    fn checked_sub(self, other: Self::Inner) -> Option<Self>;
}

impl NonZeroExt for NonZeroU64 {
    type Inner = u64;

    fn checked_sub(self, other: u64) -> Option<Self> {
        NonZeroU64::new(self.get() - other)
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union PtrRepr<T: ?Sized> {
    val: *mut T,
    sized: *mut (),
    unsize: (usize, *mut ()),
}

impl<T: ?Sized> PtrRepr<T> {
    pub fn new(val: *mut T)  -> PtrRepr<T> {
        PtrRepr { val }
    }

    pub fn from_meta_ptr(meta: usize, ptr: *mut ()) -> PtrRepr<T> {
        if mem::size_of::<*mut T>() == mem::size_of::<*mut ()>() {
            PtrRepr { sized: ptr }
        } else {
            PtrRepr { unsize: (meta, ptr) }
        }
    }

    pub fn ptr(self) -> *mut T {
        unsafe { self.val }
    }

    pub fn metadata(self) -> usize {
        if mem::size_of::<*mut T>() == mem::size_of::<*mut ()>() {
            0
        } else {
            unsafe { self.unsize.0 }
        }
    }
}
