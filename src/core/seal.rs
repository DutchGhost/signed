use core::marker::PhantomData;

#[derive(Copy, Clone)]
pub struct Contract<'id>(PhantomData<*mut &'id ()>);

impl<'id> Contract<'id> {
    #[inline(always)]
    pub(crate) const fn new() -> Self {
        Self(PhantomData)
    }
}

unsafe impl<'id> Send for Contract<'id> {}
unsafe impl<'id> Sync for Contract<'id> {}

/// This trait represents a relation from type A to type B,
/// such that type A `signs` a contract with type B.
/// # Safety
/// Must be carefull on what types this is implemented on,
/// and only ment for [`Contract`].
pub unsafe trait Signed<'id> {
    /// A new Signed type
    type Seal: for<'s> Signed<'s>;

    /// Const construction of Self
    const SELF: Self;

    /// Const construction of Self::Signed
    const SIGNED: Self::Seal;
}

unsafe impl<'id, 's> Signed<'s> for Contract<'id> {
    type Seal = Contract<'s>;
    const SELF: Self = Self::new();
    const SIGNED: Self::Seal = Contract::new();
}

/// This holds any Signed C
pub struct Seal<C: for<'s> Signed<'s>>(PhantomData<C>);

impl<C: for<'s> Signed<'s>> Seal<C> {
    #[inline(always)]
    pub(crate) fn new() -> Self {
        Self(PhantomData)
    }

    pub fn from_signed(_: C) -> Seal<C> {
        Self::new()
    }
}

impl<C: for<'s> Signed<'s>> Copy for Seal<C> {}
impl<C: for<'s> Signed<'s>> Clone for Seal<C> {
    fn clone(&self) -> Self {
        *self
    }
}
