use std::{cell::RefCell, fmt::Display};

struct DisplayIter<I, S> {
    i: I,
    s: S,
}

impl<I, S> DisplayIter<I, S> {
    pub(crate) fn new(i: I, s: S) -> Self {
        Self { i: i, s }
    }
}

impl<I, S> Iterator for DisplayIter<I, S>
where
    I: Iterator,
    I::Item: Display,
    S: Display + Sized,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.i.next()
    }
}

pub struct Displayable<I, S> {
    di: RefCell<DisplayIter<I, S>>,
}

impl<I, S> Display for Displayable<I, S>
where
    I: Iterator,
    I::Item: Display,
    S: Display + Sized,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut i = self.di.borrow_mut();

        if let Some(v) = i.next() {
            v.fmt(f)?;
        }
        while let Some(v) = i.next() {
            f.write_fmt(format_args!("{}", i.s))?;
            v.fmt(f)?;
        }
        Ok(())
    }
}

pub trait DisplayableExt<S>: Sized {
    fn into_displayable(self, separator: S) -> Displayable<Self, S>;
}

impl<I, S> DisplayableExt<S> for I
where
    I: Iterator,
    I::Item: Display,
    S: Display + Sized,
{
    fn into_displayable(self, separator: S) -> Displayable<Self, S> {
        let iter = DisplayIter::new(self, separator);
        Displayable {
            di: RefCell::new(iter),
        }
    }
}
