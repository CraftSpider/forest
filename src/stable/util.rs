use core::mem;

// TODO: Once we get custom niches, make this an enum again
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct BorrowState(usize);

impl BorrowState {
    pub fn from_val(val: usize) -> BorrowState {
        BorrowState(val)
    }

    pub fn new() -> BorrowState {
        BorrowState(0)
    }

    pub fn to_val(self) -> usize {
        self.0
    }

    #[inline]
    pub fn is_none(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn is_drop(self) -> bool {
        self.0 & 0x2 != 0
    }

    #[inline]
    pub fn is_borrow(self) -> bool {
        self.0 & 0x1 != 0
    }

    #[inline]
    pub fn is_ref(self) -> bool {
        self.0 >> 2 > 0
    }

    #[inline]
    pub fn is_mut(self) -> bool {
        self.is_borrow() && (self.0 >> 2 == 0)
    }

    #[inline]
    pub fn make_drop(self) -> BorrowState {
        BorrowState(self.0 | 0b10)
    }

    #[inline]
    pub fn incr_ref(self) -> Option<BorrowState> {
        if self.is_none() {
            Some(BorrowState(self.0 | 0b100))
        } else if self.is_ref() {
            Some(BorrowState(self.0 + 0b100))
        } else {
            None
        }
    }

    #[inline]
    pub fn decr_ref(self) -> (BorrowState, bool) {
        if self.is_ref() {
            if (self.0 >> 2) - 1 == 0 {
                (BorrowState(self.0 & 0b10), self.is_drop())
            } else {
                (BorrowState(self.0 - 0b100), false)
            }
        } else {
            (self, false)
        }
    }

    #[inline]
    pub fn incr_mut(self) -> Option<BorrowState> {
        if self.is_none() {
            Some(BorrowState(self.0 | 0b1))
        } else {
            None
        }
    }

    #[inline]
    pub fn decr_mut(self) -> (BorrowState, bool) {
        if self.is_mut() {
            (BorrowState(self.0 & 0b10), self.is_drop())
        } else {
            (self, false)
        }
    }
}

const _: () = assert!(mem::size_of::<BorrowState>() == mem::size_of::<usize>());
