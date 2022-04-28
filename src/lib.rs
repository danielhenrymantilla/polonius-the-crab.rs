#![doc = include_str!("../README.md")]
#![no_std]
#![cfg_attr(feature = "polonius",
    forbid(unsafe_code),
)]

/// ACT I SCENE I. Elsinore. A platform before the castle.
pub
mod prelude {
    pub use crate::{
        polonius,
        polonius_return,
        polonius_break,
    };
}

/// See the [top-level docs][crate] for more info.
pub
trait WithLifetime<'lt, Bound = &'lt Self> {
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
fn polonius<Ret : HKT, State : ?Sized, Err, F> (
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
        polonius_break!(42);
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
/// Sometimes, the return value of that `polonius!` closure may need to refer to
/// generic parameters. When that's the case, you'll need to tell the macro
/// about those, using `<generics…>` syntax before the `|…| ->` part of the
/// macro input:
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
    // No need to provide `K`, since not part of the return type;
    // nor to provide the bounds for `V`, except for:
    //   - nested borrows (if ever),
    //   - `?Sized` (every now and then)
    //   - or to be able to name a bound-provided associated type (super rare!)
    polonius!(<'v, V : 'v + ?Sized> |map| -> &'polonius &'v V {
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
    $(
        <
            $( $(
                $lt:lifetime
            ),+ $(,)? )?
            $( $(
                $T:ident $(:
                    $( $super:lifetime + )?
                    $( ?$Sized:ident )?
                    $( + )?
                    $( $Bound:path )?
                )?
            ),+ $(,)? )?
        >
    )?
    |$var:ident| -> $Ret:ty $body:block
) => ({
    #[allow(nonstandard_style)]
    struct __polonius_the_crab_Ret <$(
        $($(
            $lt
        ,)+)?
        $($(
            $T $(:
                $( $super + )?
                $( ?$Sized + )?
                $( $Bound )?
            )?
        ,)+)?
    )?> (
        *mut Self,
    );

    impl<
        'polonius,
        $(
            $($(
                $lt
            ,)+)?
            $($(
                $T $(:
                    $( $super + )?
                    $( ?$Sized + )?
                    $( $Bound )?
                )?
            ,)+)?
        )?
    >
        $crate::WithLifetime<'polonius>
    for
        __polonius_the_crab_Ret<$(
            $($($lt ,)+)?
            $($($T ,)+)?
        )?>
    {
        type T = $Ret;
    }

    match
        $crate::polonius::<
            __polonius_the_crab_Ret<$(
                $($($T ,)+)?
            )?>,
            _,
            _,
            _,
        >(
            $var,
            |mut $var: &mut _| {
                $var = $var;
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
macro_rules! polonius_break {( $($e:expr $(,)?)? ) => (
    return $crate::ඞ::core::result::Result::Err(($($e)?))
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
