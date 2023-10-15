#[allow(unused)]
use super::{
    exit_polonius,
    polonius,
    polonius_break,
    polonius_break_dependent,
    polonius_continue,
    polonius_loop,
    polonius_return,
    polonius_try,
};

/// Convenient entry-point to this crate's logic.
///
///   - See the [top-level docs][crate] for more info.
///
/// ## Usage
///
/**  - ```rust
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
/// They Just Work™.
///
/**  - ```rust
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
                dbg!(v);
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
) => (
    match
        $crate::polonius::<
            _,
            _,
            $crate::ForLt!(<'polonius> = $crate::ඞ::Dependent<$Ret>),
        >(
            $var,
            |mut $var: &mut _| {
                // silence the unused `mut` warning.
                #[allow(clippy::self_assignment)] {
                    $var = $var;
                }
                $crate::Either::OwnedOutput(
                    if true
                        $body
                    else {
                        // avoid a dead-code warning
                        $crate::ඞ::None.unwrap()
                    }
                )
            },
        )
    {
        | $crate::Either::BorrowingOutput(ret) => return ret.return_no_break(),
        | $crate::Either::OwnedOutput { value, input_borrow, .. } => {
            $var = input_borrow;
            value
        },
    }
)}

impl<T> ඞ::Dependent<T> {
    pub
    fn return_no_break (self)
      -> T
    {
        match self {
            | Self::Return(it) => it,
            | Self::Break(unreachable) => match unreachable {},
        }
    }
}

/// See [`polonius!`] for more info.
#[macro_export]
macro_rules! polonius_return {( $e:expr $(,)? ) => (
    return $crate::Either::BorrowingOutput($crate::ඞ::Dependent::Return($e))
)}

/// See [`polonius!`] for more info.
#[macro_export]
macro_rules! exit_polonius {( $($e:expr $(,)?)? ) => (
    return $crate::Either::OwnedOutput(
        ($($e ,)? (),).0
    )
)}

/// Perform the `?` operation (on `Result`s). See [`polonius!`] for more info.
///
/// ## Example
///
/**  - ```rust
    use {
        ::polonius_the_crab::prelude::*,
        ::std::collections::HashMap,
    };

    enum Error { /* … */ }

    fn fallible_operation (value: &'_ i32)
      -> Result<(), Error>
    {
        // …
        # Ok(())
    }

    fn get_or_insert (
        mut map: &'_ mut HashMap<i32, i32>,
    ) -> Result<&'_ i32, Error>
    {
        polonius!(|map| -> Result<&'polonius i32, Error> {
            if let Some(value) = map.get(&22) {
                // fallible_operation(value)?;
                polonius_try!(fallible_operation(value));
                polonius_return!(Ok(value));
            }
        });
        map.insert(22, 42);
        Ok(&map[&22])
    }
    ``` */
#[macro_export]
macro_rules! polonius_try {( $e:expr $(,)? ) => (
    match $e {
        | $crate::ඞ::core::result::Result::Ok(it) => it,
        | $crate::ඞ::core::result::Result::Err(err) => {
            $crate::polonius_return!(
                $crate::ඞ::core::result::Result::Err(
                    $crate::ඞ::core::convert::From::from(err),
                )
            )
        },
    }
)}

/// Convenience support for the `loop { … polonius!(…) }` pattern.
///
/// ### Example
///
/**  - ```rust
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

    // Notice how this example, *despite its usage of the fancy `.entry()` API
    // of `HashMap`s*, still needs `polonius_the_crab` to work!
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
        });
        unreachable!();
    }
    ``` */
///
/// <details class="custom"><summary>Error message without <code>::polonius_the_crab</code></summary>
///
/**  - ```rust ,compile_fail
    # compile_error!("compiler error message"); /*
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
    # */
    ``` */
///
/// ___
///
/// </details>
///
/// ## `break`s
///
/// Whilst `return` and `continue`s inside a [`polonius_loop!`] invocation are
/// quite straight-forward, `break` is actually more subtle and difficult
/// to use.
///
/// <details class="custom"><summary><span class="summary-box"><span>Click to show</span></span></summary>
///
/// Indeed, compare the `break` semantics of the following two snippets:
///
/**  - ```rust ,ignore
    let mut i = 0;
    let found = loop {
        match collection.get_mut(&i) {
            Some(entry) => if entry.is_empty() {
                break entry; // 👈
            } else {
                // …
            },
            None => i += 1,
        }
    };
    ``` */
///
/// _vs._
///
/**  - ```rust ,ignore
    let mut i = 0;
    let position = loop {
        match collection.get_mut(&i) {
            Some(entry) => if entry.is_empty() {
                break i; // 👈
            } else {
                // this requires polonius{,_the_crab} btw
                return entry;
            },
            None => i += 1,
        }
    };
    ``` */
///
/// With the former, we have a **dependent** / borrowing-from-`collection`
/// `entry` value, which is the one we want to `break`:
///
///   - this requires `polonius{,_the_crab}` (independently of the presence of
///     return-of-dependent-value statements);
///
/// Whereas with the latter, we are `break`ing `i`, an integer/index, that is,
/// a **non-dependent** value.
///
///   - this wouldn't require `polonius{,_the_crab}` if it weren't for the
///     `return entry` statement which does return a dependent item.
///
/// So, with the former, we can't use `collection` while `entry` is alive[^1] ,
/// whereas with the latter we perfectly can.
///
/// All these differences, which are type-system-based, represent information
/// which is unaccessible for the `polonius…!` family of macros, so a
/// single/unified `polonius_break!` macro for both things, for instance, would
/// be unable to make such a difference: unnecessary compile errors would then
/// ensue!
///
/// The solution is then to feature not one but _two_ `break`-ing macros,
/// depending on whether the value which we want to break **depends on/borrows**
/// the `&'polonius mut`-borrowed state.
///
///   - If yes, use [`polonius_break_dependent!`];
///
///       - this, in turn, **requires an additional `'polonius`-infected
///         `break` type annotation** for the proper lifetimes to come into play:
///
///         <code>[polonius_loop!]\(|var| -\> … <span style="color: green; font-weight: bolder;">, break: … {</span></code>
///
///   - Else, use [`polonius_break!`] \(in which case the `break` type annotation
///     should not be used\).
///
/// ### Examples
///
/**  - ```rust
    use {
        ::polonius_the_crab::{
            prelude::*,
        },
        ::std::{
            collections::HashMap,
        },
    };

    fn break_entry (mut coll: &'_ mut HashMap<i32, String>)
      -> &'_ mut String
    {
        let mut i = 0;
        //                                    vvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
        let found = polonius_loop!(|coll| -> _, break: &'polonius mut String {
            match coll.get_mut(&i) {
                Some(entry) => if entry.is_empty() {
                    polonius_break_dependent!(entry); // 👈
                } else {
                    // …
                },
                None => i += 1,
            }
        });
        found.push('!');
        found
    }
    ``` */
///
/// _vs._
///
/**  - ```rust
    use {
        ::polonius_the_crab::{
            prelude::*,
        },
        ::std::{
            collections::HashMap,
        },
    };

    fn break_index (mut coll: &'_ mut HashMap<i32, String>)
      -> &'_ mut String
    {
        let mut i = 0;
        let position = polonius_loop!(|coll| -> &'polonius mut String {
            match coll.get_mut(&i) {
                Some(entry) => if entry.is_empty() {
                    polonius_break!(i); // 👈
                } else {
                    polonius_return!(entry);
                },
                None => i += 1,
            }
        });
        // Re-using `coll` is fine if not using the `dependent` flavor of break.
        coll.get_mut(&i).unwrap()
    }
    ``` */
///
/// </details>
///
/// [^1]: In practice, with `polonius_break_dependent!` we won't be able to
/// reuse `coll` anymore in the function. If this is a problem for you, you'll
/// have no other choice but to refactor your loop into a smaller helper
/// function so as to replace that `break` with a `return`.
#[macro_export]
macro_rules! polonius_loop {(
    | $var:ident $(,)? | -> $Ret:ty $(, break: $Break:ty)?
        $body:block
    $(,)?
) => (
    loop {
        match
            $crate::polonius::<
                _,
                _,
                $crate::ForLt!(<'polonius>
                    = $crate::ඞ::Dependent< $Ret $(, $Break)? >
                ),
            >(
                &mut *$var,
                |mut $var: &mut _| {
                    // silence the unused `mut` warning.
                    #[allow(clippy::self_assignment)] {
                        $var = $var;
                    }
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
            | $crate::Either::BorrowingOutput(dependent) => match dependent {
                | $crate::ඞ::Dependent::Return(return_value) => return return_value,
                | $crate::ඞ::Dependent::Break(break_value) => $crate::ඞ::first! {
                    $((
                        break if false { loop {} } else { break_value }
                    ) (if $Break type else))? ({
                        let _: $crate::ඞ::dependent_break_without_break_ty_annotation = break_value;
                        match break_value {}
                    })
                },
            },
            | $crate::Either::OwnedOutput { value, input_borrow, .. } => {
                $var = input_borrow;
                match value {
                    | $crate::ඞ::core::ops::ControlFlow::Break(value) => {
                        break if false { loop {} } else { value };
                    },
                    | $crate::ඞ::core::ops::ControlFlow::Continue(()) => continue,
                }
            },
        }
    }
)}

/// `break` a **non-dependent value** out of a [`polonius_loop!`].
///
///   - When the value to `break` with is **a dependent value** / a value that
///     is _borrowing_ from the input, consider using
///     [`polonius_break_dependent!`] instead.
///
/// ## Example
///
/**  - ```rust
    use {
        ::std::{
            collections::HashMap,
        },
        ::polonius_the_crab::{
            *,
        },
    };

    fn example (mut map: &'_ mut HashMap<u8, i32>)
      -> Option<&'_ mut i32>
    {
        let mut i = 0;
        let x = polonius_loop!(|map| -> Option<&'polonius mut i32> {
            if let Some(entry) = map.get_mut(&i) {
                polonius_return!(Some(entry));
            }
            i += 1;
            if i == 42 {
                polonius_break!(i);
            }
        });
        assert_eq!(x, i);
        // Access to the "captured" `map` is still possible if using `polonius_break!`
        // (and thus no `break: …` annotation on the "closure")
        map.clear();
        None
    }
    ``` */
///
#[macro_export]
macro_rules! polonius_break {( $($e:expr $(,)?)? ) => (
    return $crate::Either::OwnedOutput(
        $crate::ඞ::core::ops::ControlFlow::Break(
            ($($e ,)? () ,).0
        )
    )
)}

/// `break` a **dependent value** out of a [`polonius_loop!`].
///
/// To be used in conjunction with a
/// <code>[polonius_loop!]\(|var| -\> …<span style="color: green; font-weight: bolder;">, break: …</span> {</code> invocation.
///
///   - If the `, break: …` type annotation is forgotten, then invocations to
///     `polonius_break_dependent!` will fail with an error message complaining
///     about `cannot_use__polonius_break_dependentǃ__without_a_break_type_annotation_on__polonius_loopǃ`
///
/// ## Example
///
/**  - ```rust
    use {
        ::std::{
            collections::HashMap,
        },
        ::polonius_the_crab::{
            *,
        },
    };

    fn example (mut map: &'_ mut HashMap<u8, i32>)
    {
        let mut i = 0;
        //                              needed for `polonius_break_dependent!` to work.
        //                                   vvvvvvvvvvvvvvvvvvvvvvvvvvv
        let entry = polonius_loop!(|map| -> _, break: &'polonius mut i32 {
        //                                             ^^^^^^^^^
        //                                          don't forget the special annotation either.
            if let Some(entry) = map.get_mut(&i) {
                polonius_break_dependent!(entry);
            }
            i += 1;
        });
        // `map` was consumed by the loop, and is thus unusable.
        // But the `break_dependent!`-yielded items is allowed to still be
        // borrowing it.
        *entry = 0;
    }
    ``` */
///
/// Now let's compare it to what happens when  [`polonius_break!`] is
/// (incorrectly) used in its stead:
///
/// #### Incorrect usage
///
/// The following **fails to compile**:
///
/**  - ```rust ,compile_fail
    use {
        ::std::{
            collections::HashMap,
        },
        ::polonius_the_crab::{
            *,
        },
    };

    fn example (mut map: &'_ mut HashMap<u8, i32>)
    {
        let mut i = 0;
        let entry = polonius_loop!(|map| -> _ {
            if let Some(entry) = map.get_mut(&i) {
                polonius_break!(entry);
            }
            i += 1;
        });
        *entry = 0;
    }
    ``` */
///
///    with the following error message:
///
/**    ```rust, compile_fail
    # compile_error!("compiler error message"); /*
    error: lifetime may not live long enough
      --> src/lib.rs:467:13
       |
    16 |       let entry = polonius_loop!(|map| -> _ {
       |  _________________-
    17 | |         if let Some(entry) = map.get_mut(&i) {
    18 | |             polonius_break!(entry);
       | |             ^^^^^^^^^^^^^^^^^^^^^^^ returning this value requires that `'1` must outlive `'2`
    19 | |         }
    20 | |         i += 1;
    21 | |     });
       | |      -
       | |      |
       | |______let's call the lifetime of this reference `'1`
       |        return type of closure is Result<Dependent<()>, ControlFlow<&'2 mut i32>>
    # */
    ``` */
///
///    Using `RUSTC_BOOTSTRAP=1 cargo rustc --profile-check -- -Zmacro-backtrace`
///    to "improve" the error message, we can get:
///
/**    ```rust ,compile_fail
    # compile_error!("compiler error message"); /*
    error: lifetime may not live long enough
       --> polonius-the-crab/src/lib.rs:442:12
        |
    351 |                    |mut $var: &mut _| {
        |                               -     - return type of closure is Result<Dependent<()>, ControlFlow<&'2 mut i32>>
        |                               |
        |                               let's call the lifetime of this reference `'1`
    ...
    441 |  / macro_rules! polonius_break {( $($e:expr $(,)?)? ) => (
    442 |  |     return $crate::ඞ::core::result::Result::Err(
        |  |____________^
    443 | ||         $crate::ඞ::core::ops::ControlFlow::Break(
    444 | ||             ($($e ,)? () ,).0
    445 | ||         )
    446 | ||     )
        | ||_____^ returning this value requires that `'1` must outlive `'2`
    447 |  | )}
        |  |__- in this expansion of `polonius_break!`
        |
       ::: src/lib.rs:552:13
        |
    18  |                polonius_break!(entry);
        |                ----------------------- in this macro invocation
    # */
    ``` */
///
/// Which may be a bit better at hinting that we have a borrowing problem with
/// `polonius_break!`, whereby the returned value cannot reach some borrowing /
/// lifetime requirements (those stemming from an actually-dependent break
/// value).
#[macro_export]
macro_rules! polonius_break_dependent {( $e:expr $(,)? ) => (
    return $crate::Either::BorrowingOutput(
        $crate::ඞ::Dependent::Break($e)
    )
)}

/// `continue` to the next iteration of a [`polonius_loop!`].
#[macro_export]
macro_rules! polonius_continue {() => (
    return $crate::Either::OwnedOutput(
        $crate::ඞ::core::ops::ControlFlow::<_>::Continue(())
    )
)}

// macro internals
#[doc(hidden)] /** Not part of the public API */ pub
mod ඞ {
    #![allow(nonstandard_style)]

    pub use ::core::{self, prelude::v1::*};

    pub
    enum cannot_use__polonius_break_dependentǃ__without_a_break_type_annotation_on__polonius_loopǃ
    {}

    pub
    enum Dependent<Return, Break = Never> {
        Return(Return),
        Break(Break),
    }

    use {
        cannot_use__polonius_break_dependentǃ__without_a_break_type_annotation_on__polonius_loopǃ
        as
        Never,
    };

    pub
    type dependent_break_without_break_ty_annotation =
        cannot_use__polonius_break_dependentǃ__without_a_break_type_annotation_on__polonius_loopǃ
    ;

    #[doc(hidden)] /** Not part of the public API */ #[macro_export]
    macro_rules! ඞ_first {(
        ( $($tt:tt)* )
        $($rest:tt)*
    ) => (
        $($tt)*
    )} pub use ඞ_first as first;
}
