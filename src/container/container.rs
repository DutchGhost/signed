use super::traits::{
    ContainerTrait, Contiguous, ContiguousMut, GetUnchecked, GetUncheckedMut, SplitUnchecked,
    SplitUncheckedMut,
};

use crate::core::{
    index::Index,
    proof::NonEmpty,
    range::Range,
    seal::{Contract, Seal},
};

/// A container is a generic container over type A (array).
/// it also carries a Contract (C) with it,
/// which it uses to sign contracts between the container,
/// and a range / index.
///
/// This contract is unique, and is the core trough which
/// this container can be accessed without boundschecks.
#[allow(unused)]
pub struct Container<C: for<'s> Contract<'s>, A> {
    seal: Seal<C>,
    container: A,
}

impl<C: for<'s> Contract<'s>, A: Copy> Copy for Container<C, A> {}

impl<C: for<'s> Contract<'s>, A: Clone> Clone for Container<C, A> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            seal: Seal::new(),
            container: self.container.clone(),
        }
    }
}

impl<C: for<'s> Contract<'s>, A, T> Container<C, A>
where
    A: ContainerTrait<Item = T>,
{
    /// Creates a new container from `C`.
    #[inline(always)]
    pub(crate) fn new(container: A) -> Self {
        Self {
            seal: Seal::new(),
            container,
        }
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the length of the container.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.container.base_len()
    }

    /// Returns a range into the container.
    #[inline(always)]
    pub fn range(&self) -> Range<C> {
        unsafe { Range::from_unknown(0, self.len()) }
    }

    /// Returns 2 ranges into the container,
    /// one from `0..index`, the other from `index..self.len()`.
    /// Proof `P` of the length transfers to the latter end.
    #[inline(always)]
    pub fn split_at_index<P>(&self, index: Index<C, P>) -> (Range<C>, Range<C, P>) {
        unsafe {
            (
                Range::from_unknown(0, index.integer()),
                Range::from_any(index.integer(), self.len()),
            )
        }
    }

    /// Swaps the element at index `a` with the element at index `b`.
    #[inline(always)]
    pub fn swap(&mut self, a: Index<C>, b: Index<C>)
    where
        A: GetUncheckedMut,
    {
        use core::ptr;

        unsafe {
            let self_mut = self as *mut Self;
            let pa: *mut _ = &mut (*self_mut)[a];
            let pb: *mut _ = &mut (*self_mut)[b];

            ptr::swap(pa, pb);
        }
    }

    /// Scans the range after `index`, in order from the lower indices towards the higher.
    /// While the closure returns `true`, the scan continue's, and the scanned element is included in the range.
    ///
    /// The resulting range always includes `index` in the range.
    #[inline(always)]
    pub fn scan_from<'b, F>(&'b self, index: Index<C>, mut f: F) -> Range<C, NonEmpty>
    where
        F: FnMut(&'b T) -> bool,
        T: 'b,
        A: Contiguous<Item = T>,
    {
        let mut end = index.integer();

        for item in &self[index.after()..] {
            if !f(item) {
                break;
            }

            end += 1;
        }

        end += 1;

        unsafe { Range::from_nonempty(index.integer(), end) }
    }

    /// Scans the range before `index, in order from the higher indices towards the lower.
    /// While the closure returns `true`, the scan continue's, and the scanned element is included in the range.
    ///
    /// The resulting range always includes `index` in the range.
    #[inline(always)]
    pub fn scan_from_rev<'b, F>(&'b self, index: Index<C>, mut f: F) -> Range<C, NonEmpty>
    where
        F: FnMut(&'b T) -> bool,
        T: 'b,
        A: Contiguous<Item = T>,
    {
        unsafe {
            let mut start = index.integer();

            for item in self[..index].iter().rev() {
                if !f(item) {
                    break;
                }

                start -= 1;
            }

            Range::from_nonempty(start, index.integer() + 1)
        }
    }
}

impl<C: for<'s> Contract<'s>, A, T> Container<C, A>
where
    A: SplitUnchecked<Item = T>,
{
    #[inline(always)]
    pub fn split_first(
        &self,
    ) -> Option<(
        &T,
        Container<impl for<'s> Contract<'s>, &<A as SplitUnchecked>::Split>,
    )>
    where
        <A as SplitUnchecked>::Split: GetUnchecked<Item = T>,
    {
        // The bound on <C as SplitUnchecked>::Split allows the `unchecked(0)` call.
        unsafe {
            if !self.is_empty() {
                let split = self.split_at(Index::new(1));

                let (lhs, rhs) = split;

                Some((lhs.container.unchecked(0), rhs))
            } else {
                None
            }
        }
    }

    /// Divides one container into two at `index`.
    /// The first will contain all indices from `[0, index)` and the second will contain all indices from
    /// [mid, len).
    #[inline(always)]
    pub fn split_at(
        &self,
        index: Index<C>,
    ) -> (
        Container<impl for<'s> Contract<'s>, &<A as SplitUnchecked>::Split>,
        Container<impl for<'s> Contract<'s>, &<A as SplitUnchecked>::Split>,
    ) {
        unsafe {
            let (lhs, rhs) = self.container.split_unchecked(index.integer());

            (
                Container {
                    seal: <C as Contract<'_>>::SEALED,
                    container: lhs,
                },
                Container {
                    seal: <C as Contract<'_>>::SEALED,
                    container: rhs,
                },
            )
        }
    }
}

impl<C: for<'s> Contract<'s>, A, T> Container<C, A>
where
    A: SplitUncheckedMut<Item = T>,
{
    #[inline(always)]
    pub fn split_first_mut(
        &mut self,
    ) -> Option<(
        &mut T,
        Container<impl for<'s> Contract<'s>, &mut <A as SplitUnchecked>::Split>,
    )>
    where
        <A as SplitUnchecked>::Split: GetUncheckedMut<Item = T>,
    {
        // The bound on <C as SplitUnchecked>::Split allows the `unchecked_mut(0)` call.
        unsafe {
            if !self.is_empty() {
                let split = self.split_at_mut(Index::new(1));

                let (lhs, rhs) = split;

                Some((lhs.container.unchecked_mut(0), rhs))
            } else {
                None
            }
        }
    }
    /// Divides one mutable container into two at `index`.
    /// The first will contain all indices from `[0, index)` and the second will contain all indices from
    /// [mid, len).
    #[inline(always)]
    pub fn split_at_mut(
        &mut self,
        index: Index<C>,
    ) -> (
        Container<impl for<'s> Contract<'s>, &mut <A as SplitUnchecked>::Split>,
        Container<impl for<'s> Contract<'s>, &mut <A as SplitUnchecked>::Split>,
    ) {
        unsafe {
            let (lhs, rhs) = self.container.split_unchecked_mut(index.integer());

            (
                Container {
                    seal: <C as Contract<'_>>::SEALED,
                    container: lhs,
                },
                Container {
                    seal: <C as Contract<'_>>::SEALED,
                    container: rhs,
                },
            )
        }
    }
}

use core::ops;

// &self[i]
impl<C: for<'s> Contract<'s>, A> ops::Index<Index<C>> for Container<C, A>
where
    A: GetUnchecked,
{
    type Output = A::Item;

    #[inline(always)]
    fn index(&self, index: Index<C>) -> &Self::Output {
        unsafe { self.container.unchecked(index.integer()) }
    }
}

// &mut self[i]
impl<C: for<'s> Contract<'s>, A> ops::IndexMut<Index<C>> for Container<C, A>
where
    A: GetUncheckedMut,
{
    #[inline(always)]
    fn index_mut(&mut self, index: Index<C>) -> &mut Self::Output {
        unsafe { self.container.unchecked_mut(index.integer()) }
    }
}

// &self[r]
impl<C: for<'s> Contract<'s>, A, T, P> ops::Index<Range<C, P>> for Container<C, A>
where
    A: Contiguous<Item = T>,
{
    type Output = [T];

    #[inline(always)]
    fn index(&self, r: Range<C, P>) -> &Self::Output {
        use core::slice;

        unsafe { slice::from_raw_parts(self.container.begin().offset(r.start() as isize), r.len()) }
    }
}

// &mut self[r]
impl<C: for<'s> Contract<'s>, A, P> ops::IndexMut<Range<C, P>> for Container<C, A>
where
    A: ContiguousMut,
{
    #[inline(always)]
    fn index_mut(&mut self, r: Range<C, P>) -> &mut Self::Output {
        use core::slice;

        unsafe {
            slice::from_raw_parts_mut(
                self.container.begin_mut().offset(r.start() as isize),
                r.len(),
            )
        }
    }
}

// &self[i..]
impl<C: for<'s> Contract<'s>, A, T, P> ops::Index<ops::RangeFrom<Index<C, P>>> for Container<C, A>
where
    A: Contiguous<Item = T>,
{
    type Output = [T];

    #[inline(always)]
    fn index(&self, r: ops::RangeFrom<Index<C, P>>) -> &Self::Output {
        use core::slice;

        let i = r.start.integer();

        unsafe { slice::from_raw_parts(self.container.begin().offset(i as isize), self.len() - i) }
    }
}

// &mut self[i..]
impl<C: for<'s> Contract<'s>, A, P> ops::IndexMut<ops::RangeFrom<Index<C, P>>> for Container<C, A>
where
    A: ContiguousMut,
{
    #[inline(always)]
    fn index_mut(&mut self, r: ops::RangeFrom<Index<C, P>>) -> &mut Self::Output {
        use core::slice;

        let i = r.start.integer();

        unsafe {
            slice::from_raw_parts_mut(
                self.container.begin_mut().offset(i as isize),
                self.len() - i,
            )
        }
    }
}

// &self[..i]
impl<C: for<'s> Contract<'s>, A, T, P> ops::Index<ops::RangeTo<Index<C, P>>> for Container<C, A>
where
    A: Contiguous<Item = T>,
{
    type Output = [T];

    #[inline(always)]
    fn index(&self, r: ops::RangeTo<Index<C, P>>) -> &Self::Output {
        use core::slice;

        let i = r.end.integer();

        unsafe { slice::from_raw_parts(self.container.begin(), i) }
    }
}

// &mut self[i..]
impl<C: for<'s> Contract<'s>, A, P> ops::IndexMut<ops::RangeTo<Index<C, P>>> for Container<C, A>
where
    A: ContiguousMut,
{
    #[inline(always)]
    fn index_mut(&mut self, r: ops::RangeTo<Index<C, P>>) -> &mut Self::Output {
        use core::slice;

        let i = r.end.integer();

        unsafe { slice::from_raw_parts_mut(self.container.begin_mut(), i) }
    }
}

// &self[..]
impl<C: for<'s> Contract<'s>, A, T> ops::Index<ops::RangeFull> for Container<C, A>
where
    A: Contiguous<Item = T>,
{
    type Output = [T];

    #[inline(always)]
    fn index(&self, _: ops::RangeFull) -> &Self::Output {
        self.container.as_slice()
    }
}

// &mut self[..]
impl<C: for<'s> Contract<'s>, A> ops::IndexMut<ops::RangeFull> for Container<C, A>
where
    A: ContiguousMut,
{
    fn index_mut(&mut self, _: ops::RangeFull) -> &mut Self::Output {
        self.container.as_mut_slice()
    }
}
