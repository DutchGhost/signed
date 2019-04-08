use super::traits::{
    ContainerTrait, Contiguous, ContiguousMut, GetUnchecked, GetUncheckedMut, SplitUnchecked,
    SplitUncheckedMut,
};

use crate::core::{
    index::Index,
    range::Range,
    seal::{Seal, Signed},
};

pub struct GenericContainer<C: for<'s> Signed<'s>, Container> {
    seal: Seal<C>,
    container: Container,
}

impl<C: for<'s> Signed<'s>, Container, T> GenericContainer<C, Container>
where
    Container: ContainerTrait<Item = T>,
{
    #[inline(always)]
    pub(crate) fn new(container: Container) -> Self {
        Self {
            seal: Seal::new(),
            container,
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.container.base_len()
    }

    #[inline(always)]
    pub fn range(&self) -> Range<C> {
        unsafe { Range::from(0, self.len()) }
    }
}

impl<C: for<'s> Signed<'s>, Container> GenericContainer<C, Container>
where
    Container: SplitUnchecked,
{
    #[inline(always)]
    pub fn split_at(
        &self,
        index: Index<C>,
    ) -> (
        GenericContainer<impl for<'s> Signed<'s>, &<Container as SplitUnchecked>::Split>,
        GenericContainer<impl for<'s> Signed<'s>, &<Container as SplitUnchecked>::Split>,
    ) {
        unsafe {
            let (lhs, rhs) = self.container.split_unchecked(index.integer());

            (
                GenericContainer {
                    seal: Seal::from_signed(C::SIGNED),
                    container: lhs,
                },
                GenericContainer {
                    seal: Seal::from_signed(C::SIGNED),
                    container: rhs,
                },
            )
        }
    }
}

impl<C: for<'s> Signed<'s>, Container> GenericContainer<C, Container>
where
    Container: SplitUncheckedMut,
{
    #[inline(always)]
    pub fn split_at_mut(
        &mut self,
        index: Index<C>,
    ) -> (
        GenericContainer<impl for<'s> Signed<'s>, &mut <Container as SplitUnchecked>::Split>,
        GenericContainer<impl for<'s> Signed<'s>, &mut <Container as SplitUnchecked>::Split>,
    ) {
        unsafe {
            let (lhs, rhs) = self.container.split_unchecked_mut(index.integer());

            (
                GenericContainer {
                    seal: Seal::from_signed(C::SIGNED),
                    container: lhs,
                },
                GenericContainer {
                    seal: Seal::from_signed(C::SIGNED),
                    container: rhs,
                },
            )
        }
    }
}

use core::ops;

impl<C: for<'s> Signed<'s>, Container> ops::Index<Index<C>> for GenericContainer<C, Container>
where
    Container: GetUnchecked,
{
    type Output = Container::Item;

    #[inline(always)]
    fn index(&self, index: Index<C>) -> &Self::Output {
        unsafe { self.container.unchecked(index.integer()) }
    }
}

impl<C: for<'s> Signed<'s>, Container> ops::IndexMut<Index<C>> for GenericContainer<C, Container>
where
    Container: GetUncheckedMut,
{
    #[inline(always)]
    fn index_mut(&mut self, index: Index<C>) -> &mut Self::Output {
        unsafe { self.container.unchecked_mut(index.integer()) }
    }
}

impl<C: for<'s> Signed<'s>, Container, T, P> ops::Index<Range<C, P>>
    for GenericContainer<C, Container>
where
    Container: Contiguous<Item = T>,
{
    type Output = [T];

    #[inline(always)]
    fn index(&self, r: Range<C, P>) -> &Self::Output {
        use core::slice;

        unsafe { slice::from_raw_parts(self.container.begin().offset(r.start() as isize), r.len()) }
    }
}

impl<C: for<'s> Signed<'s>, Container, P> ops::IndexMut<Range<C, P>>
    for GenericContainer<C, Container>
where
    Container: ContiguousMut,
{
    #[inline(always)]
    fn index_mut(&mut self, r: Range<C, P>) -> &mut Self::Output {
        use core::slice;

        unsafe {
            slice::from_raw_parts_mut(self.container.begin_mut().offset(r.start() as isize), r.len())
        }
    }
}
