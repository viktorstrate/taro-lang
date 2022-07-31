use std::{cell::Cell, fmt::Display, rc::Rc};

/// A generator used to generate unique references that are used to cross-reference unnamed objects
#[derive(Debug, Default, Clone)]
pub struct RefGen {
    counter: Rc<Cell<u64>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct RefID(u64);

impl Display for RefID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

impl RefGen {
    /// Make a new unique reference
    pub fn make_ref(&mut self) -> RefID {
        let counter = &*self.counter;

        let counter_val = counter.get();
        let ref_id = RefID(counter_val);
        counter.set(counter_val + 1);

        ref_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ref_gen() {
        let mut gen = RefGen::default();
        let first = gen.make_ref();

        let a = gen.clone().make_ref();
        let b = gen.clone().make_ref();

        assert_eq!(first, RefID(0));
        assert_eq!(a, RefID(1));
        assert_eq!(b, RefID(2));
    }
}
