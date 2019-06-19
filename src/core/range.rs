use core::{
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use super::{
    index::Index,
    proof::{NonEmpty, Unknown},
    seal::{Contract, Seal},
};

// Due to doc issue's, we import this.
#[allow(unused)]
use crate::container::Container;

/// This struct is range within a [`Container`].
/// It is paremeterized with a [`Contract`] `C`, to
/// guarantee uniqueness of this range, and a `Proof` `P`, which
/// is used to statically guarantee whether this range is [`NonEmpty`], or [`Unknown`].
///
/// An [`Unknown`] proof can be any range, thus also an empty range,
/// while a [`NonEmpty`] proof is guaranteed to have at least 1 element in the range.
///
/// This struct has no constructor, instead use the [`Container::range`] method
/// to obtain a range within the container.
#[allow(unused)]
pub struct Range<C: for<'s> Contract<'s>, P = Unknown> {
    start: usize,
    end: usize,
    contract: Seal<C>,
    proof: PhantomData<P>,
}

impl<C: for<'s> Contract<'s>> Range<C> {
    /// Creates a new Unknow range from `start` to `end`.
    ///
    /// # Unsafe
    /// This function is marked unsafe,
    /// because it can not be proved `start` and `end` make up a valid range.
    #[inline(always)]
    pub(crate) unsafe fn from_unknown(start: usize, end: usize) -> Range<C> {
        Range {
            start,
            end,
            contract: Seal::new(),
            proof: PhantomData,
        }
    }
}

impl<C: for<'s> Contract<'s>> Range<C, NonEmpty> {
    /// Creates a new NonEmpty range from `start` to `end`.
    /// This function is marked unsafe,
    /// because it can not be proved `start` and `end` make up a valid range.
    #[inline(always)]
    pub(crate) unsafe fn from_nonempty(start: usize, end: usize) -> Range<C, NonEmpty> {
        Range {
            start,
            end,
            contract: Seal::new(),
            proof: PhantomData,
        }
    }
}

impl<C: for<'s> Contract<'s>, P> Range<C, P> {
    /// Creates a new range from `start` to `end`.
    /// This function is marked unsafe,
    /// because it can not be proved `start` and `end` make up a valid range.
    #[inline(always)]
    pub(crate) unsafe fn from_any(start: usize, end: usize) -> Range<C, P> {
        Range {
            start,
            end,
            contract: Seal::new(),
            proof: PhantomData,
        }
    }
}

impl<C: for<'s> Contract<'s>, P> Copy for Range<C, P> {}
impl<C: for<'s> Contract<'s>, P> Clone for Range<C, P> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<C: for<'s> Contract<'s>, P, Q> PartialEq<Range<C, Q>> for Range<C, P> {
    #[inline(always)]
    fn eq(&self, other: &Range<C, Q>) -> bool {
        self.start == other.start && self.end == other.end
    }
}

impl<C: for<'s> Contract<'s>, P> Eq for Range<C, P> {}

impl<C: for<'s> Contract<'s>, P> Hash for Range<C, P> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.start.hash(h);
        self.end.hash(h);
    }
}

impl<C: for<'s> Contract<'s>, P> Range<C, P> {
    /// Returns a new NonEmpty range.
    /// Marked unsafe, because it just assumes this is called on
    /// a range that isn't empty.
    ///
    /// For the safe variant, see [`Range::nonempty`].
    #[inline]
    pub unsafe fn assume_nonempty(&self) -> Range<C, NonEmpty> {
        Range::from_nonempty(self.start(), self.end())
    }

    /// Attempts to create a NonEmpty range, returning Some on success, None on failure.
    #[inline(always)]
    pub fn nonempty(&self) -> Option<Range<C, NonEmpty>> {
        if !self.is_empty() {
            unsafe { Some(self.assume_nonempty()) }
        } else {
            None
        }
    }

    /// Returns the length of the range.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Returns `true` if the range is empty.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Returns the first index of the range.
    #[inline(always)]
    pub fn first(&self) -> Index<C, P> {
        unsafe { Index::new(self.start) }
    }

    /// Returns the middle index of the range.
    #[inline(always)]
    pub fn upper_middle(&self) -> Index<C, P> {
        let mid = self.len() / 2 + self.start;

        unsafe { Index::new(mid) }
    }

    /// Returns the start index of the range.
    #[inline(always)]
    pub fn start(&self) -> usize {
        self.start
    }

    /// Returns the end index of the range,
    #[inline(always)]
    pub fn end(&self) -> usize {
        self.end
    }

    /// Splits the range in half,
    /// with the upper middle indexing landing in the latter half.
    /// Proof `P` of the length transfers to the latter half.
    #[inline(always)]
    pub fn split_in_half(&self) -> (Range<C>, Range<C, P>) {
        let mid = (self.end - self.start) / 2 + self.start;

        unsafe {
            (
                Range::from_unknown(self.start, mid),
                Range::from_any(mid, self.end),
            )
        }
    }

    /// Splits the range at `index`.
    /// Proof `P` of the length transfers to the latter end.
    #[inline(always)]
    pub fn split_index(&self, index: Index<C>) -> (Range<C>, Range<C, P>) {
        unsafe {
            (
                Range::from_unknown(self.start, index.integer()),
                Range::from_any(index.integer(), self.end),
            )
        }
    }

    /// Returns Some if `index` is contained within the range.
    #[inline(always)]
    pub fn contains(&self, index: usize) -> Option<Index<C, P>> {
        unsafe {
            if index >= self.start && index < self.end {
                Some(Index::new(index))
            } else {
                None
            }
        }
    }
}

impl<C: for<'s> Contract<'s>> Range<C, NonEmpty> {
    /// Returns the last index of the range.
    #[inline(always)]
    pub fn last(&self) -> Index<C> {
        unsafe { Index::new(self.end - 1) }
    }

    /// Returns a new range,
    /// such that the start of the new range is incremented by one.
    #[inline(always)]
    pub fn tail(self) -> Range<C> {
        unsafe { Range::from_unknown(self.start + 1, self.end) }
    }

    /// Returns a new range,
    /// such that the end of the new range is decremented by one.
    #[inline(always)]
    pub fn head(self) -> Range<C> {
        unsafe { Range::from_unknown(self.start, self.end - 1) }
    }

    /// Advances the range backwards.
    /// Returns true if start < end after advancing.
    #[inline(always)]
    pub fn advance_back(&mut self) -> bool {
        let mut next = *self;

        next.end -= 1;
        if next.start < next.end {
            *self = next;
            true
        } else {
            false
        }
    }

    /// Advances the range forwards.
    /// Returns true if start < end after advancing.
    #[inline(always)]
    pub fn advance(&mut self) -> bool {
        let mut next = *self;

        next.start += 1;
        if next.start < next.end {
            *self = next;
            true
        } else {
            false
        }
    }
}

impl<C: for<'s> Contract<'s>, P> IntoIterator for Range<C, P> {
    type Item = Index<C>;
    type IntoIter = RangeIter<C>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        RangeIter {
            start: self.start,
            end: self.end,
            contract: Seal::new(),
        }
    }
}

/// An Iterator over any Range with contract `C`.
#[allow(unused)]
pub struct RangeIter<C: for<'s> Contract<'s>> {
    start: usize,
    end: usize,
    contract: Seal<C>,
}

impl<C: for<'s> Contract<'s>> Copy for RangeIter<C> {}
impl<C: for<'s> Contract<'s>> Clone for RangeIter<C> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<C: for<'s> Contract<'s>> Iterator for RangeIter<C> {
    type Item = Index<C>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            let idx = self.start;
            self.start += 1;
            unsafe { Some(Index::new(idx)) }
        } else {
            None
        }
    }
}

impl<C: for<'s> Contract<'s>> DoubleEndedIterator for RangeIter<C> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            self.end -= 1;
            unsafe { Some(Index::new(self.end)) }
        } else {
            None
        }
    }
}

impl<C: for<'s> Contract<'s>> core::iter::ExactSizeIterator for RangeIter<C> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.end - self.start
    }
}

#[cfg(test)]
mod tests {
    use super::Range;

    #[test]
    fn range_from() {
        use crate::core::seal::Signed;
        let range: Range<Signed> = unsafe { Range::from_unknown(0, 10) };
        assert_eq!(range.len(), 10);
    }
}
