#![forbid(unsafe_code)]

use std::convert::TryInto;

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
        loop {
            polonius! {
                |iter| -> Option<<Self as LendingIteratorItem<'polonius>>::T> {
                    if let Some(item) = iter.next() {
                        if (self.predicate)(&item) {
                            polonius_return!(Some(item));
                        }
                    } else {
                        polonius_return!(None);
                    }
                }
            }
        }
    }
}

struct WindowsMut<Slice, const WIDTH: usize> {
    slice: Slice,
    start: usize,
}

impl<'slice, T, const WIDTH: usize> WindowsMut<&'slice mut [T], WIDTH> {
    fn new (
        slice: &'slice mut [T],
    ) -> WindowsMut<&'slice mut [T], WIDTH>
    where
        WindowsMut<&'slice mut [T], WIDTH>
            : for<'n> LendingIteratorItem<'n, T = &'n mut [T; WIDTH]>
        ,
    {
        return Self { slice, start: 0 };
        // where:
        impl<'next, 'slice, T, const WIDTH: usize>
            LendingIteratorItem<'next>
        for
            WindowsMut<&'slice mut [T], WIDTH>
        {
            type T = &'next mut [T; WIDTH];
        }

        impl<'slice, T, const WIDTH: usize>
            LendingIterator
        for
            WindowsMut<&'slice mut [T], WIDTH>
        {
            fn next<'next> (
                self: &'next mut WindowsMut<&'slice mut [T], WIDTH>,
            ) -> Option<
                    &'next mut [T; WIDTH],
                >
            {
                let slice =
                    self.slice
                        .get_mut(self.start ..)?
                        .get_mut(.. WIDTH)?
                ;
                self.start += 1;
                Some(slice.try_into().unwrap())
            }
        }
    }
}

#[test]
fn test_windows_mut ()
{
    let slice = &mut [42, 0, 1, 2, 3, 4][..];
    let mut windows =
        WindowsMut::<_, 2>::new(slice)
            .filter(|&&mut [x, _]| x != 42)
    ;
    while let Some(&mut [x, ref mut y]) = windows.next() {
        *y += x;
    }
    assert_eq!(slice.last().unwrap(), &10);
}
