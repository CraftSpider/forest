use core::mem;
use core::num::NonZeroUsize;

pub(crate) const NONZERO_1: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(1) };

#[derive(Debug, Copy, Clone)]
pub(crate) enum BorrowTy {
    Mut,
    Ref(NonZeroUsize),
}

const _: () = assert!(mem::size_of::<BorrowTy>() == mem::size_of::<usize>());

#[derive(Debug, Copy, Clone)]
pub(crate) enum BorrowState {
    None,
    Borrow(BorrowTy),
    Drop(BorrowTy),
}

const _: () = assert!(mem::size_of::<BorrowState>() == mem::size_of::<usize>()*2);
