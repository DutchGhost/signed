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
                Range::from_any(index.integer(), self.len()),
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

    /// Scans the range after `index`, in order from the lower indices towards the higher.
    /// While the closure returns `true`, the scan continue's, and the scanned element is included in the range.
    ///
    /// The resulting range always includes `index` in the range.
    #[inline(always)]
    pub fn scan_from<'b, F>(&'b self, index: Index<SC>, mut f: F) -> Range<SC, NonEmpty>
    where
        F: FnMut(&'b T) -> bool,
        T: 'b,
        C: Contiguous<Item = T>,
    {
        let mut end = index.integer();

        for item in &self[index.after()..] {
            if !f(item) {
                break;
            }

            end += 1;
        }

        end += 1;

        unsafe { Range::from_ne(index.integer(), end) }
    }

    /// Scans the range before `index, in order from the higher indices towards the lower.
    /// While the closure returns `true`, the scan continue's, and the scanned element is included in the range.
    ///
    /// The resulting range always includes `index` in the range.
    #[inline(always)]
    pub fn scan_from_rev<'b, F>(&'b self, index: Index<SC>, mut f: F) -> Range<SC, NonEmpty>
    where
        F: FnMut(&'b T) -> bool,
        T: 'b,
        C: Contiguous<Item = T>,
    {
        unsafe {
            let mut start = index.integer();

            for item in self[..index].iter().rev() {
                if !f(item) {
                    break;
                }

                start -= 1;
            }

            Range::from_ne(start, index.integer() + 1)
        }
    }
}

impl<SC: for<'s> Contract<'s>, C, T> Container<SC, C>
where
    C: SplitUnchecked<Item = T>,
{
    #[inline(always)]
    pub fn split_first(&self) -> Option<(&T, Container<impl for<'s> Contract<'s>, &[T]>)>
    where
        C: GetUnchecked,
    {
        self.range().nonempty().map(|range| {
            let index = range.first();
            (
                &self[index],
                Container {
                    seal: <SC as Contract<'_>>::SEALED,
                    container: &self[index.after()..],
                },
            )
        })
    }

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

impl<SC: for<'s> Contract<'s>, C, T> Container<SC, C>
where
    C: SplitUncheckedMut<Item = T>,
{
    #[inline(always)]
    pub fn split_first_mut(
        &mut self,
    ) -> Option<(&mut T, Container<impl for<'s> Contract<'s>, &mut [T]>)>
    where
        C: SplitUnchecked<Split = [T]>,
    {
        unsafe {
            if !self.is_empty() {
                let split = self.container.split_unchecked_mut(1);

                Some((
                    split.0.unchecked_mut(0),
                    Container {
                        seal: <SC as Contract<'_>>::SEALED,
                        container: split.1,
                    },
                ))
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

/// &self[i]
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

/// &mut self[i]
impl<SC: for<'s> Contract<'s>, C> ops::IndexMut<Index<SC>> for Container<SC, C>
where
    C: GetUncheckedMut,
{
    #[inline(always)]
    fn index_mut(&mut self, index: Index<SC>) -> &mut Self::Output {
        unsafe { self.container.unchecked_mut(index.integer()) }
    }
}

/// &self[r]
impl<SC: for<'s> Contract<'s>, C, T, P> ops::Index<Range<SC, P>> for Container<SC, C>
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

/// &mut self[r]
impl<SC: for<'s> Contract<'s>, C, P> ops::IndexMut<Range<SC, P>> for Container<SC, C>
where
    C: ContiguousMut,
{
    #[inline(always)]
    fn index_mut(&mut self, r: Range<SC, P>) -> &mut Self::Output {
        use core::slice;

        unsafe {
            slice::from_raw_parts_mut(
                self.container.begin_mut().offset(r.start() as isize),
                r.len(),
            )
        }
    }
}

/// &self[i..]
impl<SC: for<'s> Contract<'s>, C, T, P> ops::Index<ops::RangeFrom<Index<SC, P>>>
    for Container<SC, C>
where
    C: Contiguous<Item = T>,
{
    type Output = [T];

    #[inline(always)]
    fn index(&self, r: ops::RangeFrom<Index<SC, P>>) -> &Self::Output {
        use core::slice;

        let i = r.start.integer();

        unsafe { slice::from_raw_parts(self.container.begin().offset(i as isize), self.len() - i) }
    }
}

///&mut self[i..]
impl<SC: for<'s> Contract<'s>, C, P> ops::IndexMut<ops::RangeFrom<Index<SC, P>>>
    for Container<SC, C>
where
    C: ContiguousMut,
{
    #[inline(always)]
    fn index_mut(&mut self, r: ops::RangeFrom<Index<SC, P>>) -> &mut Self::Output {
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

/// &self[..i]
impl<SC: for<'s> Contract<'s>, C, T, P> ops::Index<ops::RangeTo<Index<SC, P>>> for Container<SC, C>
where
    C: Contiguous<Item = T>,
{
    type Output = [T];

    #[inline(always)]
    fn index(&self, r: ops::RangeTo<Index<SC, P>>) -> &Self::Output {
        use core::slice;

        let i = r.end.integer();

        unsafe { slice::from_raw_parts(self.container.begin(), i) }
    }
}

///&mut self[i..]
impl<SC: for<'s> Contract<'s>, C, P> ops::IndexMut<ops::RangeTo<Index<SC, P>>> for Container<SC, C>
where
    C: ContiguousMut,
{
    #[inline(always)]
    fn index_mut(&mut self, r: ops::RangeTo<Index<SC, P>>) -> &mut Self::Output {
        use core::slice;

        let i = r.end.integer();

        unsafe { slice::from_raw_parts_mut(self.container.begin_mut(), i) }
    }
}

/// &self[..]
impl<SC: for<'s> Contract<'s>, C, T> ops::Index<ops::RangeFull> for Container<SC, C>
where
    C: Contiguous<Item = T>,
{
    type Output = [T];

    #[inline(always)]
    fn index(&self, _: ops::RangeFull) -> &Self::Output {
        self.container.as_slice()
    }
}

/// &mut self[..]
impl<SC: for<'s> Contract<'s>, C> ops::IndexMut<ops::RangeFull> for Container<SC, C>
where
    C: ContiguousMut,
{
    fn index_mut(&mut self, _: ops::RangeFull) -> &mut Self::Output {
        self.container.as_mut_slice()
    }
}
