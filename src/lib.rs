#![doc = include_str!("../README.md")]
#![no_std]
#![cfg_attr(feature = "polonius",
    forbid(unsafe_code),
)]

/// ACT I SCENE I. Elsinore. A platform before the castle.
pub
mod prelude {
    pub use crate::{
        exit_polonius,
        polonius,
        polonius_break,
        polonius_continue,
        polonius_loop,
        polonius_return,
    };
}

/// See the [top-level docs][crate] for more info.
pub
trait WithLifetime<'lt> { // Note: the `&'lt Self` implicit bound hack is,
                          // for once, unnecessary.
    type T;
}

/// See the [top-level docs][crate] for more info.
pub
trait HKT
where
    Self : for<'any> WithLifetime<'any>,
{}

impl<T : ?Sized> HKT for T
where
    Self : for<'any> WithLifetime<'any>,
{}

/// See the [top-level docs][crate] for more info.
pub
fn polonius<Ret : ?Sized + HKT, State : ?Sized, Err, F> (
    state: &mut State,
    branch: F,
) -> Result<
        <Ret as WithLifetime<'_>>::T,
        (&'_ mut State, Err),
    >
where
    F : FnOnce(&'_ mut State)
          -> Result<
                <Ret as WithLifetime<'_>>::T,
                Err,
            >,
{
    let err = {
        #[cfg(not(feature = "polonius"))]
        let state = unsafe {
            // SAFETY:
            // > Though this be `unsafe`, there is soundness in 't.
            //
            // More seriously, read the docs, I've detailed the reasoning there
            // in great length. And/or check the `tests/soundness.rs` test,
            // which `cargo check`s this very snippet without this `unsafe`.
            &mut *(state as *mut _)
        };
        match branch(state) {
            | Ok(ret) => return Ok(ret),
            | Err(err) => err,
        }
    };
    Err((state, err))
}

/// Convenient entry-point to this crate's logic.
///
/// ## Usage
///
/** ```rust
use ::polonius_the_crab::prelude::*;

# fn foo (arg: &mut ()) -> &mut () {
let mut a_mut_binding: &mut _ = // …
# arg;
# type SomeRetType<'__> = &'__ mut ();
# let some_cond = || true;
# let some_other_cond = || true;
# use ::core::convert::identity as stuff;

//                                      the lifetime placeholder has to be
//                                          named `'polonius` !!
//                                               vvvvvvvvv
let x = polonius!(|a_mut_binding| -> SomeRetType<'polonius> {
    let some_dependent_type = stuff(a_mut_binding);
    if some_cond() {
        polonius_return!(some_dependent_type);
    }
    if some_other_cond() {
        exit_polonius!(42);
        unreachable!();
    }
    42
});
assert_eq!(x, 42);
stuff(a_mut_binding) // macro gave it back
// …
# }
``` */
///
/// ### Generic parameters
///
/// They now Just Work™.
///
/** ```rust
use ::polonius_the_crab::prelude::*;

fn get_or_insert<'map, 'v, K, V : ?Sized> (
    mut map: &'map mut ::std::collections::HashMap<K, &'v V>,
    key: &'_ K,
    fallback_value: &'v V,
) -> &'map &'v V
where
    K : ::core::hash::Hash + Eq + Clone,
    V : ::core::fmt::Debug,
{
    polonius!(|map| -> &'polonius &'v V {
        if let Some(v) = map.get(key) {
            dbg!(v); // Even though `Debug` is not provided to the signature, it is available to the body.
            polonius_return!(v);
        }
    });
    map.insert(key.clone(), fallback_value);
    &map[key]
}
``` */
#[macro_export]
macro_rules! polonius {(
    |$var:ident $(,)?| -> $Ret:ty
        $body:block
    $(,)?
) => ({
    match
        $crate::polonius::<
            dyn for<'polonius> $crate::WithLifetime<'polonius, T = $Ret>,
            _,
            _,
            _,
        >(
            $var,
            |mut $var: &mut _| {
                // silence the unused `mut` warning.
                #[allow(clippy::self_assignment)] {
                    $var = $var;
                }
                $crate::ඞ::core::result::Result::Err(
                    if true
                        $body
                    else {
                        // avoid a dead-code warning
                        $crate::ඞ::core::option::Option::None.unwrap()
                    }
                )
            },
        )
    {
        | $crate::ඞ::core::result::Result::Ok(ret) => return ret,
        | $crate::ඞ::core::result::Result::Err((give_input_back, bail_value)) => {
            $var = give_input_back;
            bail_value
        },
    }
})}

/// See [`polonius!`] for more info.
#[macro_export]
macro_rules! polonius_return {( $e:expr $(,)? ) => (
    return $crate::ඞ::core::result::Result::Ok($e)
)}

/// See [`polonius!`] for more info.
#[macro_export]
macro_rules! exit_polonius {( $($e:expr $(,)?)? ) => (
    return $crate::ඞ::core::result::Result::Err(
        ($($e ,)? (),).0
    )
)}

/// Convenience support for the `loop { … polonius!(…) }` pattern.
///
/// ### Example
///
/** ```rust
#![forbid(unsafe_code)]
use {
    ::polonius_the_crab::{
        prelude::*,
    },
    ::std::{
        collections::HashMap,
    },
};

enum Value {
    Alive(i32),
    Daed,
}

/// Notice how this example, *despite its usage of the fancy `.entry()` API
/// of `HashMap`s*, still needs `polonius_the_crab` to express this logic!
fn get_first_alive_from_base_or_insert (
    mut map: &'_ mut HashMap<usize, Value>,
    base: usize,
    default_value: i32,
) -> &'_ i32
{
    let mut idx = base;
    // (loop {
    polonius_loop!(|map| -> &'polonius i32 {
        use ::std::collections::hash_map::*;
        // return(
        polonius_return!(
            match map.entry(idx) {
                | Entry::Occupied(entry) => match entry.into_mut() {
                    // Found a value!
                    | &mut Value::Alive(ref val) => val,
                    // "tombstone", keep searching
                    | &mut Value::Daed => {
                        idx += 1;
                        // continue;
                        polonius_continue!();
                    },
                },
                | Entry::Vacant(slot) => match slot.insert(Value::Alive(default_value)) {
                    | &mut Value::Alive(ref val) => val,
                    | &mut Value::Daed => unreachable!(),
                },
            }
        );
    })
}
``` */
///
/// <details><summary>Error message without <code>polonius</code></summary>
///
/** ```console
 error[E0499]: cannot borrow `*map` as mutable more than once at a time
   --> src/lib.rs:222:18
    |
 22 | mut map: &'_ mut HashMap<usize, Value>,
    |          - let's call the lifetime of this reference `'1`
 ...
 33 |     match map.entry(idx) {
    |           ^^^ `*map` was mutably borrowed here in the previous iteration of the loop
 ...
 45 |         | &mut Value::Alive(ref val) => val,
    |                                         --- returning this value requires that `*map` be borrowed for `'1`
``` */
///
/// ___
///
/// </details>
#[macro_export]
macro_rules! polonius_loop {(
    | $var:ident $(,)? | -> $Ret:ty
        $body:block
    $(,)?
) => (
    loop {
        match $crate::polonius!(
            | $var | -> $Ret {
                let () =
                    if true
                        $body
                    else {
                        // avoid a dead-code warning
                        $crate::ඞ::core::option::Option::None.unwrap()
                    }
                ;
                $crate::polonius_continue!();
            },
        )
        {
            | $crate::ඞ::core::ops::ControlFlow::Break(value) => break value,
            | $crate::ඞ::core::ops::ControlFlow::Continue(()) => continue,
        }
    }
)}

/// `break` a **non-dependent value** out of a [`polonius_loop!`].
///
///   - (when the value to `break` with is dependent, then the necessary pattern
///     is not `loop { polonius!(…) }` (what [`polonius_loop!`] stands for) but
///     `polonius!(… let it = loop { … break … }; …)`).
#[macro_export]
macro_rules! polonius_break {( $($e:expr $(,)?)? ) => (
    return $crate::ඞ::core::result::Result::Err(
        $crate::ඞ::core::ops::ControlFlow::Break(
            ($($e ,)? () ,).0
        )
    )
)}

/// `continue` to the next iteration of a [`polonius_loop!`].
#[macro_export]
macro_rules! polonius_continue {() => (
    return $crate::ඞ::core::result::Result::Err(
        $crate::ඞ::core::ops::ControlFlow::<_>::Continue(())
    )
)}

// macro internals
#[doc(hidden)] /** Not part of the public API */ pub
mod ඞ {
    pub use ::core; // or `std`
}

#[cfg_attr(feature = "ui-tests",
    cfg_attr(all(), doc = include_str!("compile_fail_tests.md")),
)]
mod _compile_fail_tests {}
