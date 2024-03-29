use std::{
    fmt::Debug,
    hash::Hash,
    ops::{Deref, DerefMut},
};

use super::node::identifier::Ident;

pub struct LateInit<T> {
    inner: Option<T>,
}

#[inline]
fn expect_init<U>(opt: Option<U>) -> U {
    debug_assert!(opt.is_some(), "dereferenced uninitialized LateInit value");
    opt.unwrap()
}

impl<T> LateInit<T> {
    pub fn empty() -> Self {
        LateInit { inner: None }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_none()
    }
}

impl<T: Clone> LateInit<T> {
    #[inline]
    pub fn cloned(&self) -> T {
        expect_init(self.inner.clone())
    }
}

impl<T> From<T> for LateInit<T> {
    #[inline]
    fn from(value: T) -> Self {
        LateInit { inner: Some(value) }
    }
}

impl<T> Deref for LateInit<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        expect_init(self.inner.as_ref())
    }
}

impl<T> DerefMut for LateInit<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        expect_init(self.inner.as_mut())
    }
}

impl<T> AsRef<T> for LateInit<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        expect_init(self.inner.as_ref())
    }
}

impl<T> AsMut<T> for LateInit<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        expect_init(self.inner.as_mut())
    }
}

impl<'a> Into<Ident<'a>> for LateInit<Ident<'a>> {
    #[inline]
    fn into(self) -> Ident<'a> {
        expect_init(self.inner)
    }
}

impl<T> Default for LateInit<T> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<T: Debug> Debug for LateInit<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            Some(val) => val.fmt(f),
            None => f.write_str("<<UNINITIALIZED>>"),
        }
    }
}

impl<T: Clone> Clone for LateInit<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Copy> Copy for LateInit<T> {}

impl<T: PartialEq> PartialEq for LateInit<T> {
    fn eq(&self, other: &Self) -> bool {
        *expect_init(self.inner.as_ref()) == *expect_init(other.inner.as_ref())
    }
}

impl<T: Eq> Eq for LateInit<T> {}

impl<T: Hash> Hash for LateInit<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}
