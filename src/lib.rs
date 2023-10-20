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

#[doc(no_inline)]
pub use ::higher_kinded_types::ForLt;

pub use macros::ඞ;
mod macros;

mod r#try;

/// See the [top-level docs][crate] for more info.
pub
fn polonius<'i, Input : ?Sized, OwnedOutput, BorrowingOutput : ?Sized> (
    input_borrow: &'i mut Input,
    branch:
        impl for<'any>
            FnOnce(&'any mut Input)
              -> PoloniusResult<
                    BorrowingOutput::Of<'any>,
                    OwnedOutput,
                >
    ,
) -> PoloniusResult<
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
        | PoloniusResult::Borrowing(dependent) => {
            PoloniusResult::Borrowing(dependent)
        },
        | PoloniusResult::Owned { value, .. } => {
            PoloniusResult::Owned {
                value,
                input_borrow,
            }
        },
    }
}

/// Placeholder type to be used when _constructing_ a
/// [`PoloniusResult::Owned`].
///
/// [`PoloniusResult::Owned`]: type@PoloniusResult::Owned
///
/// Since there is no access to the original `input_borrow` yet (due to the
/// polonius limitation), this [`Placeholder`] is used in its stead.
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
///     | Some(ret) => PoloniusResult::Borrowing(ret),
///     | None => PoloniusResult::Owned {
///         value: 42,
///         input_borrow: /* WHAT TO PUT HERE?? */ Placeholder, // 👈👈
///     },
/// }) {
///     // `polonius` magic
///     | PoloniusResult::Borrowing(dependent_entry) => {
///         // ...
///         stuff(dependent_entry);
///     },
///     | PoloniusResult::Owned {
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
/// Hence the [`PoloniusResult::Owned()`] constructor shorthand,
/// so as to be able to write:
///
/// ```rust
/// # const _IGNORED: &str = stringify! {
/// PoloniusResult::Owned(42)
/// // is just a shorthand for:
/// PoloniusResult::Owned {
///     value: 42,
///     input_borrow: /* WHAT TO PUT HERE?? */ Placeholder, // 👈👈
/// }
/// # };
/// ```
pub
struct Placeholder;

pub enum PoloniusResult<BorrowingOutput, OwnedOutput, InputBorrow = Placeholder> {
    Borrowing(BorrowingOutput),
    Owned {
        value: OwnedOutput,
        input_borrow: InputBorrow,
    },
}

impl<BorrowingOutput, OwnedOutput> PoloniusResult<BorrowingOutput, OwnedOutput> {
    /// Tuple-variant-looking sugar to _construct_
    /// the <code>[Self::Owned]</code> variant.
    ///
    /// It's just convenience sugar for
    /// <code>[Self::Owned] { value, input_borrow: [Placeholder] }</code>.
    ///
    /// ```rust
    /// # const _IGNORED: &str = stringify! {
    /// PoloniusResult::Owned(42)
    /// // is the same as:
    /// PoloniusResult::Owned {
    ///     value: 42,
    ///     input_borrow: Placeholder,
    /// }
    /// # };
    /// ```
    ///
    /// See [`Placeholder`] for more info.
    ///
    /// [Self::Owned]: type@Self::Owned
    #[allow(nonstandard_style)]
    pub
    const
    fn Owned(value: OwnedOutput)
      -> Self
    {
        Self::Owned {
            value: value,
            input_borrow: Placeholder,
        }
    }
}

#[cfg_attr(feature = "ui-tests",
    cfg_attr(all(), doc = include_str!("compile_fail_tests.md")),
)]
mod _compile_fail_tests {}
