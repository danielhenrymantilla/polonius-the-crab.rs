#![forbid(unsafe_code)]

use ::polonius_the_crab::prelude::*;

trait LendingIteratorItem<'n, Bound = &'n mut Self> {
    type T;
}

trait LendingIterator : for<'n> LendingIteratorItem<'n> {
    fn next (self: &'_ mut Self)
      -> Option< <Self as LendingIteratorItem<'_>>::T >
    ;

    fn filter<P> (
        self,
        predicate: P,
    ) -> Filter<Self, P>
    where
        Self : Sized,
        P : FnMut(&'_ <Self as LendingIteratorItem<'_>>::T) -> bool,
    {
        Filter {
            iter: self,
            predicate,
        }
    }
}

pub
struct Filter<I, P> {
    iter: I,
    predicate: P,
}

impl<'n, I, P> LendingIteratorItem<'n> for Filter<I, P>
where
    I : LendingIterator,
    P : FnMut(&'_ <I as LendingIteratorItem<'_>>::T) -> bool,
{
    type T = <I as LendingIteratorItem<'n>>::T;
}

impl<I, P> LendingIterator for Filter<I, P>
where
    I : LendingIterator,
    P : FnMut(&'_ <I as LendingIteratorItem<'_>>::T) -> bool,
{
    fn next (self: &'_ mut Self)
      -> Option< <Self as LendingIteratorItem<'_>>::T >
    {
        let mut iter = &mut self.iter;
        while polonius! {
            <I : LendingIterator>
            |iter| -> Option< <I as LendingIteratorItem<'polonius>>::T > {
                if let Some(item) = iter.next() {
                    if (self.predicate)(&item) {
                        polonius_return!(Some(item));
                    } else {
                        true
                    }
                } else {
                    false
                }
            }
        } {}
        return None;
    }
}
