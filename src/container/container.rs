use super::traits::{
    ContainerTrait, Contiguous, ContiguousMut, GetUnchecked, GetUncheckedMut, SplitUnchecked,
    SplitUncheckedMut,
};

use crate::core::{
    index::Index,
    range::Range,
    seal::{Seal, Contract},
};

/// A container is a generic container over type C.
/// it also carries a Signed-Contract SC with it,
/// which it uses to sign contracts between the container,
/// and a range / index.
/// 
/// This contract is unique, and is the core trough which
/// this container can be accessed without boundschecks.
#[allow(unused)]
pub struct Container<SC: for<'s> Contract<'s>, C> {
    seal: Seal<SC>,
    container: C,
}

impl<SC: for<'s> Contract<'s>, C, T> Container<SC, C>
where
    C: ContainerTrait<Item = T>,
{
    /// Creates a new container from `C`.
    #[inline(always)]
    pub(crate) fn new(container: C) -> Self {
        Self {
            seal: Seal::new(),
            container,
        }
    }

    /// Returns the length of the container.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.container.base_len()
    }

    /// Returns a range into the container.
    #[inline(always)]
    pub fn range(&self) -> Range<SC> {
        unsafe { Range::from(0, self.len()) }
    }

    /// Returns 2 ranges into the container,
    /// one from `0..index`, the other from `index..self.len()`.
    /// Proof `P` of the length transfers to the latter end.
    #[inline(always)]
    pub fn split_at_index<P>(&self, index: Index<SC, P>) -> (Range<SC>, Range<SC, P>) {
        unsafe {
            (
                Range::from(0, index.integer()),
                Range::from_any(index.integer(), self.len())
            )
        }
    }

    /// Swaps the element at index `a` with the element at index `b`.
    #[inline(always)]
    pub fn swap(&mut self, a: Index<SC>, b: Index<SC>)
    where
        C: GetUncheckedMut,
    {
        use core::ptr;

        unsafe {
            let self_mut = self as *mut Self;
            let pa: *mut _ = &mut (*self_mut)[a];
            let pb: *mut _ = &mut (*self_mut)[b];

            ptr::swap(pa, pb);
        }
    }
}

impl<SC: for<'s> Contract<'s>, C> Container<SC, C>
where
    C: SplitUnchecked,
{
    /// Divides one container into two at `index`.
    /// The first will contain all indices from `[0, index)` and the second will contain all indices from
    /// [mid, len).
    #[inline(always)]
    pub fn split_at(
        &self,
        index: Index<SC>,
    ) -> (
        Container<impl for<'s> Contract<'s>, &<C as SplitUnchecked>::Split>,
        Container<impl for<'s> Contract<'s>, &<C as SplitUnchecked>::Split>,
    ) {
        unsafe {
            let (lhs, rhs) = self.container.split_unchecked(index.integer());

            (
                Container {
                    seal: <SC as Contract<'_>>::SEALED,
                    container: lhs,
                },
                Container {
                    seal: <SC as Contract<'_>>::SEALED,
                    container: rhs,
                },
            )
        }
    }
}

impl<SC: for<'s> Contract<'s>, C> Container<SC, C>
where
    C: SplitUncheckedMut,
{
    /// Divides one mutable container into two at `index`.
    /// The first will contain all indices from `[0, index)` and the second will contain all indices from
    /// [mid, len).
    #[inline(always)]
    pub fn split_at_mut(
        &mut self,
        index: Index<SC>,
    ) -> (
        Container<impl for<'s> Contract<'s>, &mut <C as SplitUnchecked>::Split>,
        Container<impl for<'s> Contract<'s>, &mut <C as SplitUnchecked>::Split>,
    ) {
        unsafe {
            let (lhs, rhs) = self.container.split_unchecked_mut(index.integer());

            (
                Container {
                    seal: <SC as Contract<'_>>::SEALED,
                    container: lhs,
                },
                Container {
                    seal: <SC as Contract<'_>>::SEALED,
                    container: rhs,
                },
            )
        }
    }
}

use core::ops;

impl<SC: for<'s> Contract<'s>, C> ops::Index<Index<SC>> for Container<SC, C>
where
    C: GetUnchecked,
{
    type Output = C::Item;

    #[inline(always)]
    fn index(&self, index: Index<SC>) -> &Self::Output {
        unsafe { self.container.unchecked(index.integer()) }
    }
}

impl<SC: for<'s> Contract<'s>, C> ops::IndexMut<Index<SC>> for Container<SC, C>
where
    C: GetUncheckedMut,
{
    #[inline(always)]
    fn index_mut(&mut self, index: Index<SC>) -> &mut Self::Output {
        unsafe { self.container.unchecked_mut(index.integer()) }
    }
}

impl<SC: for<'s> Contract<'s>, C, T, P> ops::Index<Range<SC, P>>
    for Container<SC, C>
where
    C: Contiguous<Item = T>,
{
    type Output = [T];

    #[inline(always)]
    fn index(&self, r: Range<SC, P>) -> &Self::Output {
        use core::slice;

        unsafe { slice::from_raw_parts(self.container.begin().offset(r.start() as isize), r.len()) }
    }
}

impl<SC: for<'s> Contract<'s>, C, P> ops::IndexMut<Range<SC, P>>
    for Container<SC, C>
where
    C: ContiguousMut,
{
    #[inline(always)]
    fn index_mut(&mut self, r: Range<SC, P>) -> &mut Self::Output {
        use core::slice;

        unsafe {
            slice::from_raw_parts_mut(self.container.begin_mut().offset(r.start() as isize), r.len())
        }
    }
}
