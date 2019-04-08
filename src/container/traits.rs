/// The most basic container.
/// The container hold elements of type `Item`.
///
/// This container can have indices and ranges that are in bounds.
pub unsafe trait ContainerTrait {
    type Item;

    /// Returns the length of the container.
    fn base_len(&self) -> usize;
}

unsafe impl<'a, C: ?Sized + ContainerTrait> ContainerTrait for &'a C {
    type Item = C::Item;

    #[inline(always)]
    fn base_len(&self) -> usize {
        (**self).base_len()
    }
}

unsafe impl<'a, C: ?Sized + ContainerTrait> ContainerTrait for &'a mut C {
    type Item = C::Item;

    #[inline(always)]
    fn base_len(&self) -> usize {
        (**self).base_len()
    }
}

pub unsafe trait Contiguous: ContainerTrait {
    fn begin(&self) -> *const Self::Item;
    fn end(&self) -> *const Self::Item;
    fn as_slice(&self) -> &[Self::Item];
}

unsafe impl<'a, C: ?Sized + Contiguous> Contiguous for &'a C {
    #[inline(always)]
    fn begin(&self) -> *const Self::Item {
        (**self).begin()
    }

    #[inline(always)]
    fn end(&self) -> *const Self::Item {
        (**self).end()
    }

    #[inline(always)]
    fn as_slice(&self) -> &[Self::Item] {
        (**self).as_slice()
    }
}

unsafe impl<'a, C: ?Sized + Contiguous> Contiguous for &'a mut C {
    #[inline(always)]
    fn begin(&self) -> *const Self::Item {
        (**self).begin()
    }

    #[inline(always)]
    fn end(&self) -> *const Self::Item {
        (**self).end()
    }

    #[inline(always)]
    fn as_slice(&self) -> &[Self::Item] {
        (**self).as_slice()
    }
}

pub unsafe trait ContiguousMut: Contiguous {
    #[inline(always)]
    fn begin_mut(&mut self) -> *mut Self::Item {
        self.begin() as *mut _
    }

    #[inline(always)]
    fn end_mut(&mut self) -> *mut Self::Item {
        self.end() as *mut _
    }

    fn as_mut_slice(&mut self) -> &mut [Self::Item];
}

unsafe impl<'a, C: ?Sized + ContiguousMut> ContiguousMut for &'a mut C {
    #[inline(always)]
    fn as_mut_slice(&mut self) -> &mut [Self::Item] {
        (**self).as_mut_slice()
    }
}

pub unsafe trait GetUnchecked: ContainerTrait {
    #[inline(always)]
    unsafe fn unchecked(&self, index: usize) -> &Self::Item;
}

unsafe impl<'a, C: ?Sized + GetUnchecked> GetUnchecked for &'a C {
    #[inline(always)]
    unsafe fn unchecked(&self, index: usize) -> &Self::Item {
        (**self).unchecked(index)
    }
}

unsafe impl<'a, C: ?Sized + GetUnchecked> GetUnchecked for &'a mut C {
    #[inline(always)]
    unsafe fn unchecked(&self, index: usize) -> &Self::Item {
        (**self).unchecked(index)
    }
}

pub unsafe trait GetUncheckedMut: GetUnchecked {
    #[inline(always)]
    unsafe fn unchecked_mut(&mut self, index: usize) -> &mut Self::Item;
}

unsafe impl<'a, C: ?Sized + GetUncheckedMut> GetUncheckedMut for &'a mut C {
    #[inline(always)]
    unsafe fn unchecked_mut(&mut self, index: usize) -> &mut Self::Item {
        (**self).unchecked_mut(index)
    }
}

pub unsafe trait SplitUnchecked: Contiguous {
    type Split: ?Sized;
    unsafe fn split_unchecked(&self, index: usize) -> (&Self::Split, &Self::Split);
}

unsafe impl<'a, C: ?Sized + SplitUnchecked> SplitUnchecked for &'a C {
    type Split = C::Split;
    unsafe fn split_unchecked(&self, index: usize) -> (&Self::Split, &Self::Split) {
        (**self).split_unchecked(index)
    }
}

unsafe impl<'a, C: ?Sized + SplitUnchecked> SplitUnchecked for &'a mut C {
    type Split = C::Split;
    unsafe fn split_unchecked(&self, index: usize) -> (&Self::Split, &Self::Split) {
        (**self).split_unchecked(index)
    }
}

pub unsafe trait SplitUncheckedMut: SplitUnchecked + ContiguousMut {
    unsafe fn split_unchecked_mut(&mut self, index: usize) -> (&mut Self::Split, &mut Self::Split);
}

unsafe impl<'a, C: ?Sized + SplitUncheckedMut> SplitUncheckedMut for &'a mut C {
    unsafe fn split_unchecked_mut(&mut self, index: usize) -> (&mut Self::Split, &mut Self::Split) {
        (**self).split_unchecked_mut(index)
    }
}
unsafe impl<T> ContainerTrait for [T] {
    type Item = T;

    #[inline(always)]
    fn base_len(&self) -> usize {
        self.len()
    }
}

unsafe impl<T> Contiguous for [T] {
    #[inline(always)]
    fn begin(&self) -> *const Self::Item {
        self.as_ptr()
    }

    #[inline(always)]
    fn end(&self) -> *const Self::Item {
        unsafe { self.begin().offset(self.len() as isize) }
    }

    #[inline(always)]
    fn as_slice(&self) -> &[Self::Item] {
        self
    }
}

unsafe impl<T> ContiguousMut for [T] {
    #[inline(always)]
    fn as_mut_slice(&mut self) -> &mut [Self::Item] {
        self
    }
}

unsafe impl<T> GetUnchecked for [T] {
    #[inline(always)]
    unsafe fn unchecked(&self, index: usize) -> &Self::Item {
        self.get_unchecked(index)
    }
}

unsafe impl<T> GetUncheckedMut for [T] {
    #[inline(always)]
    unsafe fn unchecked_mut(&mut self, index: usize) -> &mut Self::Item {
        self.get_unchecked_mut(index)
    }
}

unsafe impl<T> SplitUnchecked for [T] {
    type Split = [T];

    unsafe fn split_unchecked(&self, index: usize) -> (&Self::Split, &Self::Split) {
        (self.get_unchecked(..index), self.get_unchecked(index..))
    }
}

unsafe impl<T> SplitUncheckedMut for [T] {
    unsafe fn split_unchecked_mut(&mut self, index: usize) -> (&mut Self::Split, &mut Self::Split) {
        let len = self.len();

        let ptr = self.as_mut_ptr();

        (
            core::slice::from_raw_parts_mut(ptr, index),
            core::slice::from_raw_parts_mut(ptr.add(index), len - index),
        )
    }
}
unsafe impl<T> ContainerTrait for Vec<T> {
    type Item = T;

    #[inline(always)]
    fn base_len(&self) -> usize {
        self.len()
    }
}

unsafe impl<T> Contiguous for Vec<T> {
    #[inline(always)]
    fn begin(&self) -> *const Self::Item {
        self.as_ptr()
    }

    #[inline(always)]
    fn end(&self) -> *const Self::Item {
        unsafe { self.begin().offset(self.len() as isize) }
    }

    #[inline(always)]
    fn as_slice(&self) -> &[Self::Item] {
        self
    }
}

unsafe impl<T> ContiguousMut for Vec<T> {
    #[inline(always)]
    fn as_mut_slice(&mut self) -> &mut [Self::Item] {
        self
    }
}

unsafe impl<T> GetUnchecked for Vec<T> {
    #[inline(always)]
    unsafe fn unchecked(&self, index: usize) -> &Self::Item {
        self.get_unchecked(index)
    }
}

unsafe impl<T> GetUncheckedMut for Vec<T> {
    #[inline(always)]
    unsafe fn unchecked_mut(&mut self, index: usize) -> &mut Self::Item {
        self.get_unchecked_mut(index)
    }
}
