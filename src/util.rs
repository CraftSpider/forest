use core::num::{NonZeroU64, NonZeroUsize, NonZeroIsize};

pub trait NonZeroExt: Sized {
    type Inner;

    fn checked_sub(self, other: Self::Inner) -> Option<Self>;
}

impl NonZeroExt for NonZeroU64 {
    type Inner = u64;

    fn checked_sub(self, other: Self::Inner) -> Option<Self> {
        NonZeroU64::new(self.get() - other)
    }
}

impl NonZeroExt for NonZeroUsize {
    type Inner = usize;

    fn checked_sub(self, other: Self::Inner) -> Option<Self> {
        NonZeroUsize::new(self.get() - other)
    }
}

impl NonZeroExt for NonZeroIsize {
    type Inner = isize;

    fn checked_sub(self, other: Self::Inner) -> Option<Self> {
        NonZeroIsize::new(self.get() - other)
    }
}
