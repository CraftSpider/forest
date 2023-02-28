use core::fmt::Debug;
use core::mem::MaybeUninit;
use core::slice::SliceIndex;
use core::ptr;
use core::ops::{Deref, DerefMut, Index, IndexMut};
use crate::util::{MaybeUninitArray, MaybeUninitSlice};

pub struct ArrayVec<T, const N: usize> {
    init: usize,
    data: [MaybeUninit<T>; N],
}

impl<T, const N: usize> ArrayVec<T, N> {
    pub const fn new() -> ArrayVec<T, N> {
        ArrayVec {
            init: 0,
            data: MaybeUninit::UNINIT,
        }
    }

    pub const fn len(&self) -> usize {
        self.init
    }

    pub const fn capacity(&self) -> usize {
        N
    }

    pub const fn is_empty(&self) -> bool {
        self.init == 0
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { MaybeUninitSlice::assume_init_ref(&self.data[..self.init]) }
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe { MaybeUninitSlice::assume_init_mut(&mut self.data[..self.init]) }
    }

    /// # Panics
    ///
    /// If a push would overflow the capacity of the backing array
    pub fn push(&mut self, item: T) {
        if self.init >= N {
            panic!("ArrayVec is full")
        }
        self.data[self.init].write(item);
        self.init += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.init == 0 {
            None
        } else {
            self.init -= 1;
            let mu = unsafe { ptr::read(&self.data[self.init]) };
            Some(unsafe { mu.assume_init() })
        }
    }

    pub fn get<I: SliceIndex<[T]>>(&self, idx: I) -> Option<&I::Output> {
        self.as_slice().get(idx)
    }

    pub fn get_mut<I: SliceIndex<[T]>>(&mut self, idx: I) -> Option<&mut I::Output> {
        self.as_slice_mut().get_mut(idx)
    }
}

impl<T, const N: usize> Deref for ArrayVec<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const N: usize> DerefMut for ArrayVec<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}

impl<T, const N: usize> AsRef<[T]> for ArrayVec<T, N> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, const N: usize> AsMut<[T]> for ArrayVec<T, N> {
    fn as_mut(&mut self) -> &mut [T] {
        self.as_slice_mut()
    }
}

impl<T, I, const N: usize> Index<I> for ArrayVec<T, N>
where
    I: SliceIndex<[T]> + Debug + Clone,
{
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.as_slice().get(index.clone())
            .unwrap_or_else(|| panic!("Index {:?} out of bounds for ArrayVec", index))
    }
}

impl<T, I, const N: usize> IndexMut<I> for ArrayVec<T, N>
where
    I: SliceIndex<[T]> + Debug + Clone,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.as_slice_mut().get_mut(index.clone())
            .unwrap_or_else(|| panic!("Index {:?} out of bounds for ArrayVec", index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push() {
        let mut v = ArrayVec::<_, 5>::new();
        assert_eq!(v.len(), 0);
        assert_eq!(v.as_slice(), &[]);
        assert_eq!(v.get(0), None);
        v.push(0);
        assert_eq!(v.len(), 1);
        assert_eq!(v.as_slice(), &[0]);
        assert_eq!(v.get(0), Some(&0));
    }

    #[test]
    #[should_panic = "ArrayVec is full"]
    fn test_push_capacity() {
        let mut v = ArrayVec::<_, 1>::new();
        v.push(0);
        v.push(1);
    }

    #[test]
    fn test_pop() {
        let mut v = ArrayVec::<_, 5>::new();
        v.push(0);
        v.push(1);
        v.push(2);
        v.push(3);
        v.push(4);
        assert_eq!(v.as_slice(), &[0, 1, 2, 3, 4]);

        assert_eq!(v.pop(), Some(4));
        assert_eq!(v.as_slice(), &[0, 1, 2, 3]);
        assert_eq!(v.pop(), Some(3));
        assert_eq!(v.as_slice(), &[0, 1, 2]);
        assert_eq!(v.pop(), Some(2));
        assert_eq!(v.as_slice(), &[0, 1]);
        assert_eq!(v.pop(), Some(1));
        assert_eq!(v.as_slice(), &[0]);
        assert_eq!(v.pop(), Some(0));
        assert_eq!(v.as_slice(), &[]);
        assert_eq!(v.pop(), None);
    }
}
