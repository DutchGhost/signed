use core::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use super::{
    proof::{NonEmpty},
    seal::{Seal, Contract},
};

#[allow(unused)]
pub struct Index<C: for<'s> Contract<'s>, P = NonEmpty> {
    index: usize,

    /// A sealed contract
    contract: Seal<C>,

    /// A proof P over the length.
    proof: PhantomData<P>,
}

impl<C: for<'s> Contract<'s>, P> Index<C, P> {
    /// Creates a new Index from `index`.
    /// This function is marked unsafe,
    /// because `index` is not known to be valid.
    #[inline(always)]
    pub unsafe fn new(index: usize) -> Index<C, P> {
        Self {
            index,
            contract: Seal::new(),
            proof: PhantomData,
        }
    }

    /// Returns the index as an integer offset.
    #[inline(always)]
    pub fn integer(&self) -> usize {
        self.index
    }
}

impl<C: for<'s> Contract<'s>, P> Copy for Index<C, P> {}
impl<C: for<'s> Contract<'s>, P> Clone for Index<C, P> {
    fn clone(&self) -> Self {
        *self
    }
}

/// Index can only be compared with other indices of the same branding
impl<C: for<'s> Contract<'s>, P, Q> PartialEq<Index<C, Q>> for Index<C, P> {
    #[inline(always)]
    fn eq(&self, rhs: &Index<C, Q>) -> bool {
        self.index == rhs.index
    }
}

impl<C: for<'s> Contract<'s>, P> Eq for Index<C, P> {}

impl<C: for<'s> Contract<'s>, P, Q> PartialOrd<Index<C, Q>> for Index<C, P> {
    #[inline(always)]
    fn partial_cmp(&self, rhs: &Index<C, Q>) -> Option<Ordering> {
        Some(self.index.cmp(&rhs.index))
    }

    #[inline(always)]
    fn lt(&self, rhs: &Index<C, Q>) -> bool {
        self.index < rhs.index
    }
}

impl<C: for<'s> Contract<'s>, P> Ord for Index<C, P> {
    #[inline(always)]
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.index.cmp(&rhs.index)
    }
}

impl<C: for<'s> Contract<'s>, P> Hash for Index<C, P> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.index.hash(h)
    }
}
