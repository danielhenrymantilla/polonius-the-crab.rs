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

/// The key stone of the API of this crate.
/// See the [top-level docs][crate] for more info.
///
/// Signature formatted for readability:
/**
```rust
# const _IGNORED: &str = stringify! {
fn polonius<'i, Input : ?Sized, OwnedOutput, BorrowingOutput : ?Sized> (
    input_borrow: &'i mut Input,
    branch:
        impl for<'any>
            FnOnce(&'any mut Input)
              -> PoloniusResult<
                    BorrowingOutput::Of<'any>,
                    OwnedOutput, // -----------+
                >                           // |
    ,                                       // | `polonius()`
) -> PoloniusResult<                        // | magic
        BorrowingOutput::Of<'i>,            // | adds "back"
        OwnedOutput, &'i mut Input, // <-------+ the `input_borrow`
    >
where
    BorrowingOutput : ForLt,
# };
``` */
///
/// ## Turbofishing a `ForLt!()` parameter.
///
/// As described in the [top-level docs][crate], the key aspect that allows this
/// function to be both _sound_, and _generic_, is that it involves a
/// `for<'any>`-quantified lifetime in its [branching][PoloniusResult] closure,
/// and a lifetime-generic generic type parameter.
///
/// There was no stutter: this is indeed a generic generic type parameter: the
/// API is said to be "higher-kinded", related to HKTs ([higher-kinded
/// types][hkts]).
///
/// Hence the <code>BorrowingOutput : [ForLt]</code> (lifetime-generic) generic
/// type parameter.
///
/// [ForLt]: trait@ForLt
///
/// Such "`For` types" involved in these HKT APIs, however, **cannot be
/// elided**, since they do not play well with **type inference**.
///
/// This means that turbofishing this _third_ type parameter is:
///   - mandatory;
///   - to be done using the [`ForLt!`] macro.
///
/// ```rust
/// # /*
/// polonius::<_, _, ForLt!(…)>(…)
/// # */
/// ```
///
/// [hkts]: https://docs.rs/higher-kinded-types
///
/// If this sounds too complex or abstract, know that there also are:
///
/// ## Easier APIs for the most pervasive use cases
///
/// These are provided as the macros that accompany this crate:
///
///   - [`polonius!`] for most cases;
///
///   - [`polonius_loop!`] as extra sugar for a specific shape of
///     `loop { polonius!(…) }`
///
///       - ⚠️ [not every `loop { polonius!(…); … }` case can be translated][l]
///         to a `polonius_loop!`. When in doubt, fall back to a lower-level
///         [`polonius!`] invocation, or even to a manual [`polonius()`] call.
///
/// [l]: https://github.com/danielhenrymantilla/polonius-the-crab.rs/issues/11
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
        OwnedOutput, &'i mut Input,
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
    let owned_value = match branch(tentative_borrow) {
        | PoloniusResult::Borrowing(dependent) => {
            return PoloniusResult::Borrowing(dependent);
        },
        | PoloniusResult::Owned { value, .. } => value,
    }; // <- `drop(PoloniusResult::Owned { .. })`.
       // See https://github.com/rust-lang/rust/issues/126520 for more info.
    PoloniusResult::Owned {
        value: owned_value,
        input_borrow,
    }
}

/// Placeholder type to be used when _constructing_ a
/// [`PoloniusResult::Owned`].
///
/// [`PoloniusResult::Owned`]: type@PoloniusResult::Owned
///
/// Since there is no access to the original `input_borrow` yet (due to the
/// very polonius limitation that makes this crate necessary), this
/// [`Placeholder`] is used in its stead.
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

/// Output type of both the [`polonius()`] function and the closure it takes.
///
/// It represents the conceptual code-flow / branch disjunction between:
///
///   - having some _dependent_ type still [`Borrowing`][Self::Borrowing] from
///     the `input_borrow` _given_/forfeited to [`polonius()`];
///
///   - or having no such type, _i.e._, having only "[`Owned`][type@Self::Owned]"
///     (as far as the `input_borrow` is concerned) output.
///
///       - [`polonius()`] magic makes it so, in this branch/case, its
///         `.input_borrow` shall be populated for the caller to get back access
///         to it.
pub
enum PoloniusResult<BorrowingOutput, OwnedOutput, InputBorrow = Placeholder> {
    /// Variant to return in the "happy" case where our tentative (re)borrow
    /// actually yielded some dependent / still-`Borrowing` type which we wish
    /// to keep hold of, _e.g._, [to `return` it from the
    /// function][`polonius_return!`].
    Borrowing(BorrowingOutput),

    /// Variant to return in the branches where we are done with any dependent
    /// value, and we would like to instead get our `input_borrow` back.
    Owned {
        /// The inner `value` of the [`Self::Owned(value)`][Self::Owned()] case.
        value: OwnedOutput,

        /// When constructing this variant, inside [`polonius()`]' closure,
        /// a [`Placeholder`] is to be put in its stead, to let [`polonius()`]
        /// replace it with its `input_borrow`.
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
