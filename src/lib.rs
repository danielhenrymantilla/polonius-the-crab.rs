#![doc = include_str!("../README.md")]
#![no_std]
#![cfg_attr(feature = "polonius",
    forbid(unsafe_code),
)]
#![allow(uncommon_codepoints)]

/// ACT I SCENE I. Elsinore. A platform before the castle.
pub
mod prelude {
    pub use crate::{
        exit_polonius,
        polonius,
        polonius_break,
        polonius_break_dependent,
        polonius_continue,
        polonius_loop,
        polonius_return,
        polonius_try,
    };
}

pub use macros::ඞ;
mod macros;

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

#[cfg_attr(feature = "ui-tests",
    cfg_attr(all(), doc = include_str!("compile_fail_tests.md")),
)]
mod _compile_fail_tests {}
