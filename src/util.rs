use core::num::{NonZeroU64, NonZeroUsize, NonZeroIsize};
use std::mem::MaybeUninit;

pub trait MaybeUninitArray<T, const N: usize>: Sized {
    const UNINIT: [Self; N];
}

pub trait MaybeUninitSlice<T>: Sized {
    unsafe fn assume_init_ref(val: &[Self]) -> &[T];
    unsafe fn assume_init_mut(val: &mut [Self]) -> &mut [T];
}

impl<T, const N: usize> MaybeUninitArray<T, N> for MaybeUninit<T> {
    const UNINIT: [Self; N] = unsafe { MaybeUninit::uninit().assume_init() };
}

impl<T> MaybeUninitSlice<T> for MaybeUninit<T> {
    unsafe fn assume_init_ref(val: &[Self]) -> &[T] {
        unsafe { &*(val as *const [Self] as *const [T]) }
    }

    unsafe fn assume_init_mut(val: &mut [Self]) -> &mut [T] {
        unsafe { &mut *(val as *mut [Self] as *mut [T]) }
    }
}

pub trait NonZeroExt: Sized {
    type Inner;

    fn checked_add(self, other: Self::Inner) -> Option<Self>;
    fn checked_sub(self, other: Self::Inner) -> Option<Self>;
}

impl NonZeroExt for NonZeroU64 {
    type Inner = u64;

    fn checked_add(self, other: Self::Inner) -> Option<Self> {
        Self::checked_add(self, other)
    }

    fn checked_sub(self, other: Self::Inner) -> Option<Self> {
        NonZeroU64::new(self.get().checked_sub(other)?)
    }
}

impl NonZeroExt for NonZeroUsize {
    type Inner = usize;

    fn checked_add(self, other: Self::Inner) -> Option<Self> {
        Self::checked_add(self, other)
    }

    fn checked_sub(self, other: Self::Inner) -> Option<Self> {
        NonZeroUsize::new(self.get().checked_sub(other)?)
    }
}

impl NonZeroExt for NonZeroIsize {
    type Inner = isize;

    fn checked_add(self, other: Self::Inner) -> Option<Self> {
        NonZeroIsize::new(self.get().checked_add(other)?)
    }

    fn checked_sub(self, other: Self::Inner) -> Option<Self> {
        NonZeroIsize::new(self.get().checked_sub(other)?)
    }
}
