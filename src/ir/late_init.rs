use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use super::node::identifier::Ident;

pub struct LateInit<T> {
    inner: Option<T>,
}

fn expect_init<U>(opt: Option<U>) -> U {
    debug_assert!(opt.is_some(), "dereferenced uninitialized LateInit value");
    unsafe { opt.unwrap_unchecked() }
}

impl<T> LateInit<T> {
    pub fn empty() -> Self {
        LateInit { inner: None }
    }
}

impl<T> From<T> for LateInit<T> {
    fn from(value: T) -> Self {
        LateInit { inner: Some(value) }
    }
}

impl<T> Deref for LateInit<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        expect_init(self.inner.as_ref())
    }
}

impl<T> DerefMut for LateInit<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        expect_init(self.inner.as_mut())
    }
}

impl<T> AsRef<T> for LateInit<T> {
    fn as_ref(&self) -> &T {
        expect_init(self.inner.as_ref())
    }
}

impl<T> AsMut<T> for LateInit<T> {
    fn as_mut(&mut self) -> &mut T {
        expect_init(self.inner.as_mut())
    }
}

impl<'a> Into<Ident<'a>> for LateInit<Ident<'a>> {
    fn into(self) -> Ident<'a> {
        expect_init(self.inner)
    }
}

impl<T: Default> Default for LateInit<T> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<T: Debug> Debug for LateInit<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LateInit")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T: Clone> Clone for LateInit<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
