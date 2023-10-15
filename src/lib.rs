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

        Either,
        ForLt,
    };
}

#[doc(no_inline)]
pub use ::higher_kinded_types::ForLt;

pub use macros::ඞ;
mod macros;

/// See the [top-level docs][crate] for more info.
pub
fn polonius<'i, Input : ?Sized, OwnedOutput, BorrowingOutput : ?Sized> (
    input_borrow: &'i mut Input,
    branch:
        impl for<'any>
            FnOnce(&'any mut Input)
              -> Either<
                    BorrowingOutput::Of<'any>,
                    OwnedOutput,
                >
    ,
) -> Either<
        BorrowingOutput::Of<'i>,
        OwnedOutput,
        &'i mut Input,
    >
where
    BorrowingOutput : ForLt,
{
    #[cfg(feature = "polonius")]
    let tentative_borrow = &mut *input_borrow;
    #[cfg(not(feature = "polonius"))]
    let tentative_borrow = unsafe {
        // SAFETY:
        // > Though this be `unsafe`, there is soundness in 't.
        //
        // More seriously, read the docs, I've detailed the reasoning there
        // in great length. And/or check the `tests/soundness.rs` test,
        // which `cargo check`s this very snippet without this `unsafe`.
        &mut *(input_borrow as *mut _)
    };
    match branch(tentative_borrow) {
        | Either::BorrowingOutput(dependent) => {
            Either::BorrowingOutput(dependent)
        },
        | Either::OwnedOutput { value, .. } => {
            Either::OwnedOutput {
                value,
                input_borrow,
            }
        },
    }
}

/// Placeholder type to be used when _constructing_ an [`Either::OwnedOutput`]:
///
/// [`Either::OwnedOutput`]: type@Either::OwnedOutput
///
/// there is no access to the original `input_borrow` yet (due to the polonius
/// limitation, so this [`Placeholder`] is used in its stead.
///
/// ```rust, no_run
/// use ::polonius_the_crab::*;
///
/// type StringRef = ForLt!(&str);
///
/// let map: &mut ::std::collections::HashMap<i32, String> = // ...
/// # None.unwrap();
///
/// # use drop as stuff;
/// #
/// match polonius::<_, _, StringRef>(map, |map| match map.get(&22) {
///     | Some(ret) => Either::BorrowingOutput(ret),
///     | None => Either::OwnedOutput {
///         value: 42,
///         input_borrow: /* WHAT TO PUT HERE?? */ Placeholder, // 👈👈
///     },
/// }) {
///     // `polonius` magic
///     | Either::BorrowingOutput(dependent_entry) => {
///         // ...
///         stuff(dependent_entry);
///     },
///     | Either::OwnedOutput {
///         value,
///         input_borrow: map, // we got our `map` borrow back! 🙌
///     } => {
///         assert_eq!(value, 42);
///         stuff(map);
///     },
/// }
/// ```
///
/// Note that despite intellectually interesting w.r.t properly understanding
/// the API, providing that `input_borrow: Placeholder` does not provide any
/// valuable information to the call, and is thus rather noisy.
///
/// Hence the [`Either::OwnedOutput()`] constructor shorthand,
/// so as to be able to write:
///
/// ```rust
/// # const _IGNORED: &str = stringify! {
/// Either::OwnedOutput(42)
/// // instead of
/// Either::OwnedOutput {
///     value: 42,
///     input_borrow: /* WHAT TO PUT HERE?? */ Placeholder, // 👈👈
/// }
/// # };
/// ```
pub
struct Placeholder;

pub enum Either<BorrowingOutput, OwnedOutput, InputBorrow = Placeholder> {
    BorrowingOutput(BorrowingOutput),
    OwnedOutput {
        value: OwnedOutput,
        input_borrow: InputBorrow,
    },
}

impl<BorrowingOutput, OwnedOutput> Either<BorrowingOutput, OwnedOutput> {
    /// Tuple-variant-looking constructor sugar to _construct_
    /// the <code>[Self::OwnedOutput]</code> variant.
    ///
    /// It's just convenience sugar for
    /// <code>[Self::OwnedOutput] { value, input_borrow: [Placeholder] }</code>.
    ///
    /// ```rust
    /// # const _IGNORED: &str = stringify! {
    /// Either::OwnedOutput(42)
    /// // is the same as:
    /// Either::OwnedOutput {
    ///     value: 42,
    ///     input_borrow: Placeholder,
    /// }
    /// # };
    /// ```
    ///
    /// See [`Placeholder`] for more info.
    ///
    /// [Self::OwnedOutput]: type@Self::OwnedOutput
    #[allow(nonstandard_style)]
    pub
    const
    fn OwnedOutput(value: OwnedOutput)
      -> Self
    {
        Self::OwnedOutput {
            value: value,
            input_borrow: Placeholder,
        }
    }
}

#[cfg_attr(feature = "ui-tests",
    cfg_attr(all(), doc = include_str!("compile_fail_tests.md")),
)]
mod _compile_fail_tests {}
