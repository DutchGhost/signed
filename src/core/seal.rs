use core::marker::PhantomData;

#[derive(Copy, Clone)]
pub struct Signed<'a>(PhantomData<*mut &'a ()>);

impl<'a> Signed<'a> {
    #[inline(always)]
    #[allow(unused)]
    pub(crate) const fn new() -> Self {
        Self(PhantomData)
    }
}

unsafe impl<'a> Send for Signed<'a> {}
unsafe impl<'a> Sync for Signed<'a> {}

/// This trait represents a contract between type A<'a>, and type A<'b>,
/// such that a constant Seal<A<'b>> can be created from A<'a>.
///
/// This trait is marked unsafe, because both lifetimes 'a and 'b must be invariant.
pub unsafe trait Contract<'a> {
    /// A new Signed type
    type With: for<'s> Contract<'s>;

    /// Const construction of Self::Signed
    const SEALED: Seal<Self::With>;
}

unsafe impl<'a, 'b> Contract<'b> for Signed<'a> {
    type With = Signed<'b>;

    /// Const construction of Seal<Self::With>
    const SEALED: Seal<Self::With> = Seal(PhantomData);
}

unsafe impl <'a, 'b, C: ?Sized + Contract<'b>> Contract<'b> for &'a C {
    type With = C::With;

    const SEALED: Seal<Self::With> = C::SEALED;
}

unsafe impl <'a, 'b, C: ?Sized + Contract<'b>> Contract<'b> for &'a mut C {
    type With = C::With;

    const SEALED: Seal<Self::With> = C::SEALED;
}

/// A seal over any contract `C`.
pub struct Seal<C: for<'s> Contract<'s>>(PhantomData<C>);

impl<C: for<'s> Contract<'s>> Seal<C> {
    #[inline(always)]
    pub(crate) fn new() -> Self {
        Self(PhantomData)
    }
}

impl<C: for<'s> Contract<'s>> Copy for Seal<C> {}
impl<C: for<'s> Contract<'s>> Clone for Seal<C> {
    fn clone(&self) -> Self {
        *self
    }
}
