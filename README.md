### _Though this be madness, yet **there is method** in 't._

![stable-rust-stands-atop-dead-zpolonius](https://user-images.githubusercontent.com/9920355/165785441-e0573795-81d8-4273-bede-c2d5f9e7fa55.png)

<!-- ![stable-rust-stands-atop-dead-zpolonius](https://user-images.githubusercontent.com/9920355/165641079-e9987007-a088-4d9f-bdbe-7042cf3b3f02.png)
-->

<details class="custom"><summary>More context</summary>

 1. **Hamlet**:

    > _For yourself, sir, shall grow old as I am – if, like a crab, you could go backward._

 1. **Polonius**:

    > _Though this be madness, yet **there is method** in 't._

 1. **Polonius**, eventually:

    ![polonius-lying-dead](https://user-images.githubusercontent.com/9920355/165641368-b0e3590c-3dce-45ce-af07-aa8addabd666.png)

</details>

# `::polonius-the-crab`

Tools to feature more lenient Polonius-based borrow-checker patterns in stable Rust.

[![Repository](https://img.shields.io/badge/repository-GitHub-brightgreen.svg)](
https://github.com/danielhenrymantilla/polonius-the-crab.rs)
[![Latest version](https://img.shields.io/crates/v/polonius-the-crab.svg)](
https://crates.io/crates/polonius-the-crab)
[![Documentation](https://docs.rs/polonius-the-crab/badge.svg)](
https://docs.rs/polonius-the-crab)
[![MSRV](https://img.shields.io/badge/MSRV-1.67.0-white)](
https://gist.github.com/danielhenrymantilla/9b59de4db8e5f2467ed008b3c450527b)
[![unsafe internal](https://img.shields.io/badge/unsafe-internal-important.svg)](
https://github.com/rust-secure-code/safety-dance/)
[![no_std compatible](https://img.shields.io/badge/no__std-compatible-success.svg)](
https://github.com/rust-secure-code/safety-dance/)
[![License](https://img.shields.io/crates/l/polonius-the-crab.svg)](
https://github.com/danielhenrymantilla/polonius-the-crab.rs/blob/master/LICENSE-ZLIB)
[![CI](https://github.com/danielhenrymantilla/polonius-the-crab.rs/workflows/CI/badge.svg)](
https://github.com/danielhenrymantilla/polonius-the-crab.rs/actions)

<!-- Templated by `cargo-generate` using https://github.com/danielhenrymantilla/proc-macro-template -->

## Rationale: limitations of the NLL borrow checker

See the following issues:

  - [#54663 – Borrow checker extends borrow range in code with early return](
    https://github.com/rust-lang/rust/issues/54663)

  - [#70255 – Weird error for mutable references in a loop](
    https://github.com/rust-lang/rust/issues/70255)

  - [#92985 – Filter adapter for LendingIterator requires Polonius](
    https://github.com/rust-lang/rust/issues/92985) (this one marks bonus
    points for involving GATs and the pervasive `LendingIterator` example).

  - [_etc._](https://github.com/rust-lang/rust/labels/NLL-polonius)

All these examples boil down to the following canonical instance:

```rust ,compile_fail
#![forbid(unsafe_code)]
use ::std::{
    collections::HashMap,
};

/// Typical example of lack-of-Polonius limitation: get_or_insert pattern.
/// See https://nikomatsakis.github.io/rust-belt-rust-2019/#72
fn get_or_insert (
    map: &'_ mut HashMap<u32, String>,
) -> &'_ String
{
    if let Some(v) = map.get(&22) {
        return v;
    }
    map.insert(22, String::from("hi"));
    &map[&22]
}
```

<details class="custom"><summary>error message</summary>

```rust
# /*
 error[E0502]: cannot borrow `*map` as mutable because it is also borrowed as immutable
  --> src/lib.rs:53:5
   |
14 |     map: &mut HashMap<u32, String>,
   |          - let's call the lifetime of this reference `'1`
15 | ) -> &String {
16 |     if let Some(v) = map.get(&22) {
   |                      --- immutable borrow occurs here
17 |         return v;
   |                - returning this value requires that `*map` be borrowed for `'1`
18 |     }
19 |     map.insert(22, String::from("hi"));
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
# */
```

</details>

## Explanation

<details open class="custom"><summary><span class="summary-box"><span>Click to hide</span></span></summary>

Now, this pattern is known to be sound / a false positive from the current
borrow checker, NLL.

  - The technical reason behind it is the _named_ / in-function-signature
    lifetime involved in the borrow: contrary to a fully-in-body anonymous
    borrow, borrows that last for a "named" / outer-generic lifetime are deemed
    to last _until the end of the function_, **across all possible codepaths**
    (even those unreachable whence the borrow starts).

      - a way to notice this difference is to, when possible, rewrite the function
        as a macro. By virtue of being syntactically inlined, it will involve
        anonymous lifetimes and won't cause any trouble.

### Workarounds

So "jUsT uSe UnSaFe" you may suggest. But this is tricky:

  - does your use-case _really_ fit this canonical example?

      - or a variant: will it still fit it as the code evolves / in face of
        code refactorings?

  - even when we know "we can use `unsafe`", actually using it is subtle and
    error-prone. Since `&mut` borrows are often involved in this situation,
    one may accidentally end up transmuting a `&` reference to a `&mut`
    reference, which is _always_ UB.

  - both of these issues lead to a certain completely legitimate allergy to
    `unsafe`_code, and the very reassuring
    `#![forbid(unsafe_code)]`-at-the-root-of-the-crate pattern.

### Non-`unsafe` albeit cumbersome workarounds for lack-of-Polonius issues

<details class="custom"><summary><span class="summary-box"><span>Click to show</span></span></summary>

  - if possible, **reach for a dedicated API**.
    For instance, the `get_or_insert()` example can be featured using the
    `.entry()` API:

    ```rust
    #![forbid(unsafe_code)]
    use ::std::{
        collections::HashMap,
    };

    fn get_or_insert (
        map: &'_ mut HashMap<u32, String>,
    ) -> &'_ String
    {
        map.entry(22).or_insert_with(|| String::from("hi"))
    }
    ```

    Sadly, the reality is that you won't always have such convenient APIs at
    your disposal.

  - otherwise, you can perform successive non-idiomatic lookups to avoid
    holding the borrow for too long:

    ```rust
    #![forbid(unsafe_code)]
    use ::std::{
        collections::HashMap,
    };

    fn get_or_insert (
        map: &'_ mut HashMap<u32, String>,
    ) -> &'_ String
    {
        // written like this to show the "transition path" from previous code
        let should_insert =
            if let Some(_discarded) = map.get(&22) {
                false
            } else {
                true
            }
        ;
        // but `should_insert` can obviously be shortened down to `map.get(&22).is_none()`
        // or, in this very instance, to `map.contains_key(&22).not()`.
        if should_insert {
            map.insert(22, String::from("hi"));
        }
        map.get(&22).unwrap() // or `&map[&22]`
    }
    ```

  - finally, related to the "this only happens with concrete named lifetimes"
    issue, a clever non-`unsafe` albeit cumbersome way to circumvent the
    limitation is to use [CPS / callbacks / a scoped API](
    https://docs.rs/with_locals):

    ```rust
    #![forbid(unsafe_code)]
    use ::std::{
        collections::HashMap,
    };

    fn with_get_or_insert<R> (
        map: &'_ mut HashMap<u32, String>,
        yield_:     impl FnOnce(
    /* -> */ &'_ String
                    ) -> R ) -> R
    {
        if let Some(v) = map.get(&22) {
            yield_(v)
        } else {
            map.insert(22, String::from("hi"));
            yield_(&map[&22])
        }
    }
    ```

While you should try these workarounds first and see how they apply to your
codebase, sometimes they're not applicable or way too cumbersome compared to
"a tiny bit of `unsafe`".

In that case, as with all the cases of known-to-be-sound `unsafe` patterns, the
ideal solution is to factor it out down to its own small and easy to review
crate or module, and then use the non-`unsafe fn` API thereby exposed 👌.

</details>

### Enters `::polonius-the-crab`

![polonius-the-crab](https://user-images.githubusercontent.com/9920355/165791136-26d09367-3d84-4d09-8f6a-6a3dd91ffc50.jpg)
<!-- ![polonius the crab](https://user-images.githubusercontent.com/9920355/165742437-d644851e-83c3-45c7-941f-c7909cab0192.png) -->

#### Explanation of its implementation

<details class="custom"><summary><span class="summary-box"><span>Click to show</span></span></summary>

So, back to that "safety encapsulation" idea:

 1. let's find a canonical instance of this borrow checker issue that is known
    to be sound and accepted under Polonius;

 1. and tweak it so that it can then be re-used as a general-purpose tool for
    _most_ of these issues.

And if we stare at the borrow checker issues above, we can see there are two
defining ingredients:

  - An explicit generic lifetime parameter (potentially elided);
  - **A branch**, where one of the branches returns based on that borrow, whilst
    the other is no longer interested in it.

The issue is then that that second branch ought to get back access to the
stuff borrowed in the first branch, but the current borrow checker denies it.

That's where we'll sprinkle some correctly-placed `unsafe` to make the "borrow
checker look the other way" just for a moment, the right moment.

This thus gives us (in pseudo-code first):

```rust ,ignore
fn polonius<'r, T> (
    borrow: &'r mut T,
    branch:
        impl // generic type to apply to all possible scopes.
        for<'any> // <- higher-order lifetime ensures the `&mut T` infected with it…
        FnOnce(&'any mut T)      // …can only escape the closure…
                    // vvvv        … through its return type and its return type only.
          -> Option< _<'any> > // <- The `Some` / `None` discriminant represents the branch info.
                  // ^^^^^^^
                  // some return type allowed to depend on `'any`.
                  // For instance, in the case of `get_or_insert`, this could
                  // have been `&'any String` (or `Option<&'any String>`).
                  // Bear with me for the moment and tolerate this pseudo-code.
    ,
) -> Result< // <- we "forward the branch", but with data attached to the fallback one (`Err(…)`).
        _<'r>, // <- "plot twist": `'any` above was `'r` !
        &'r mut T, // <- through Arcane Magic™ we get to transmute the `None` into an `Err(borrow)`
    >
{
    let tentative_borrow = &mut *borrow; // reborrow
    if let Some(dependent) = branch(tentative_borrow) {
        /* within this branch, the reborrow needs to last for `'r` */
        return Ok(dependent);
    }
    /* but within this branch, the reborrow needs to have ended: only Polonius supports that kind of logic */

    // give the borrow back
    Err(borrow) // <- without Polonius this is denied
}
```

This function, ignoring that generic unspecified `_<'…>` return type in
pseudo-code, does indeed represent a canonical example of the borrow checker
issue (without `-Zpolonius`, it will reject the `Err(borrow)` line saying that
`borrow` needs to be borrowed for `'r` so that `dependent` is, and that `'r`
spans until _any_ end of function (the borrow checker bug).

Whereas with `-Zpolonius` it is accepted.

  - [Demo](https://rust.godbolt.org/z/81sn7oK9s)

#### The ArcaneMagic™

The correct use of `unsafe`, here, to palliate the lack of `-Zpolonius`, is to
change:

```rust ,ignore
let tentative_borrow = &mut *borrow; // reborrow
```

into:

```rust ,ignore
let tentative_borrow = unsafe { &mut *(borrow as *mut _) }; // reborrow
```

where `unsafe { &mut *(thing as *mut _) }` is the canonical way to perform
**lifetime**(-of-the-borrow) **extension**: the lifetime of that `&mut` borrow
is then no longer tied, in any way, to `'r` nor to `*borrow`.

  - Some of you might have been tempted to use `mem::transmute`. While that does
    indeed work, it is a strictly more flexible API, which in the case of
    `unsafe`, means it's a strictly more dangerous API. With `transmute`, for
    instance, when the borrowee has lifetime parameters of its own, those may
    be erased as well, whereas a downgrade-to-pointer-and-upgrade-back-to-ref
    operation is guaranteed to "erase" only the outer lifetime of the borrow,
    leaving the inner type untouched: definitely safer.

**The borrow checker no longer holds our hand**, as far as overlapped usage of
`borrow` and `tentative_borrow` is concerned (which would be UB). **It is now
up to us to ensure _no runtime path_ can ever lead to such borrows
overlapping**.

And indeed they don't, as the simple branch showcases:

  - in the `Some` branch,
    the `dependent` is still borrowing `tentative_borrow`, and thus, `*borrow`. But
    we do not use `borrow` anymore in that branch, _nor in the caller's body_, as
    long as dependent is used. Indeed, signature-wise, we do tell that that
    `dependent` return value, of type `_<'r>`, is borrowing from `*borrow`, due to
    that repetition of the `'r` name.

  - in the `None` branch,
    there is no `dependent`, and `tentative_borrow` isn't used anymore, so it is
    sound to refer to `borrow` again.

In other words:

> _Though this be `unsafe`, yet **there is soundness** in 't._

As an extra precaution, this crate does even guard that usage of `unsafe`
through a `cfg`-opt-out, so that when using `-Zpolonius`, the `unsafe` is
removed, and yet the body of the function, as well as its signature, compiles
fine (this is further enforced in CI through a special `test`).

#### Generalizing it

##### `Option<T<'_>>` becomes `Either<T<'_>, U>`

It turns out that we don't have to restrict the `branch` to returning no data on
`None`, and that we can use it as a "channel" through which to smuggle
**non-borrowing** data.

This leads to replacing `Option< T<'any> >` with `Either< T<'any>, U > `

  - Notice how the `U` cannot depend on `'any` since it can't name it
    (generic parameter introduced _before_ the `'any` quantification ever gets
    introduced).

##### The `FnOnceReturningAnOption` trick is replaced with a `HKT` pattern

  - (where `FnOnceReturningAnOption` is the helper trait used in the `Demo`
    snippet above)

Indeed, a `FnOnceReturningAnOption`-based signature would be problematic on the
caller's side, since:

  - **it _infers_ the higher-order-`'any`-infected return type of the closure
    through the actual closure instance being fed**;

  - **but a closure only gets to be higher-order when the API it is fed to
    _explicitly requires it to_**

      - see <https://docs.rs/higher-order-closure> for more info.

So this leads to a situation where both the caller and callee expect each other
to disambiguate what the higher-order return value of the closure should be,
leading to no higher-orderness to begin with and/or to type inference errors.

  - Note that the `hrtb!` macro from <https://docs.rs/higher-order-closure>, or
    the actual `for<…>`-closures RFC such crate polyfills, would help in that
    regard. But the usage then becomes, imho, way more convoluted than any of
    the aforementioned workarounds, defeating the very purpose of this crate.

So that `Ret<'any>` is achieved in another manner. Through HKTs, that is, through
"generic generics" / "generics that are, themselves, generic":

```rust ,ignore
//! In pseudo-code:
fn polonius<'r, T, Ret : <'_>> (
    borrow: &'r mut T,
    branch: impl FnOnce(&'_ mut T) -> Either<Ret<'_>, ()>,
) -> Either<
        Ret<'r>,
        (), &'r mut T,
    >
```

This cannot directly be written in Rust, but you can define a trait representing
the `<'_>`-ness of a type (`HKT` in this crate), and with it, use
`as WithLifetime<'a>::T` as the "feed `<'a>`" operator:

```rust
// Real code!
use ::polonius_the_crab::{HKT, WithLifetime};

fn polonius<'r, T, Ret : HKT> (
    borrow: &'r mut T,
    branch: impl FnOnce(&'_ mut T) -> Either< <Ret as WithLifetime<'_>>::T, () >,
) -> Either<
        <Ret as WithLifetime<'r>>::T,
        (), &'r mut T,
    >
# { unimplemented!(); }
```

We have reached the definition of the actual `fn polonius` exposed by this very
crate!

Now, a `HKT` type is still cumbersome to use. If we go back to that
`get_or_insert` example that was returning a `&'_ String`, we'd need to express
that "generic type" representing `<'lt> => &'lt String`, such as:

```rust
# use ::polonius_the_crab::WithLifetime;
#
/// Pseudo-code (`StringRefNaïve` is not a type, `StringRefNaïve<'…>` is).
type StringRefNaïve<'any> = &'any String;

/// Real HKT code: make `StringRef` a fully-fledged stand-alone type
struct StringRef;
/// And now express the `<'lt> => &'lt String` relationship:
impl<'lt> WithLifetime <'lt>
   for StringRef // is:  ⇓
{                     // ⇓
                      // ⇓
    type T =         &'lt String    ;
}
```

#### New: the `dyn for<'a>` _ad-hoc_ HKT trick

<details class="custom"><summary><span class="summary-box"><span>Click to show</span></span></summary>

Actually, as of `0.2.0`, this crate now uses a fancier trick, which stems from
the following observation. Consider the type
`dyn for<'any> WithLifetime<'any, T = &'any String>`:

  - It's a standalone/in-and-of-itself type
    (which `type StringRef<'any> = &'any String` (without `'any`) is not).

  - And yet thanks to that `for<'any> … T = &'any String` quantification,
    it does manage to express that nested / currified type-level function
    wherein we can feed any `'lt` and get a `&'lt String` back.

That is, it achieves the same as our
`struct StringRef; impl<'lt> WithLifetime<'lt> for StringRef` definition!

But with no need to define an extra type, that is, in an _ad-hoc_ / pluggable
manner, which incidentally allows getting rid of the need to specify the
generics in scope.

  - For instance, expressing the `'lt => &'lt T` HKT for some generic `T` in
    scope can simply be done with `dyn for<'lt> WithLifetime<'lt, T = &'lt T>`,
    whereas with the hand-rolled approach it requires writing:

    ```rust
    // That extra parameter achieves a `where Self : 'lt` implicit bound on the
    // universally quantified `'lt`.
    trait WithLifetime<'lt, WhereSelfIsUsableWithinLtHack = &'lt Self> {
        type T : ?Sized;
    }

    struct Ref<T>(T);

    impl<'lt, T> WithLifetime<'lt> for Ref<T> {
        type T = &'lt T;
    }
    ```

      - moreover, the `WhereSelfIsUsableWithinLtHack` is not even necessary
        when using the `dyn for<'lt> WithLifetime<'lt, T = &'lt T>` approach:
        neat!

</details>

### Putting it altogether: `get_or_insert` with no `.entry()` nor double-lookup

</details>

So this crate exposes a "raw" `polonius()` function that has the `unsafe` in its
body, and which is quite powerful at tackling these lack-of-polonius related
issues.

```rust
use ::polonius_the_crab::{polonius, Either, WithLifetime};

#[forbid(unsafe_code)] // No unsafe code in this function: VICTORY!!
fn get_or_insert (
    map: &'_ mut ::std::collections::HashMap<i32, String>,
) -> &'_ String
{
    type StringRef = dyn for<'lt> WithLifetime<'lt, T = &'lt String>;

    match polonius::<_, _, StringRef>(map, |map| match map.get(&22) {
        | Some(ret) => Either::BorrowingOutput(ret),
        | None => Either::OwnedOutput(()),
    }) {
        | Either::BorrowingOutput(ret) => {
            // no second-lookup!
            ret
        },
        // we get the borrow back (we had to give the original one to `polonius()`)
        | Either::OwnedOutput { value: (), input_borrow: map, .. } => {
            map.insert(22, String::from("…"));
            &map[&22]
        },
    }
}
```

  - [Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=8d5bd3271604a02508587b3c0b964d79)

We'll have to admit this is **quite cumbersome to use!** 😵‍💫

Hence why this crate also offers:

## Convenient macros for ergonomic usage 😗👌

Mainly, the `polonius!` entry point, within which you can use `polonius_return!`
to **early return the dependent value**, or `exit_polonius!` to instead
"break" / leave the `polonius!` block with a **non-dependent** value (notice how
the _branch_ nature of this borrow checker limitation is kept in the very bones
of the API).

  - The `polonius!` macro requires that a `'polonius`-infected return type be
    used —the HKT marker (`for<'polonius>`), for those having followed the
    implementation.

This leads to the following `get_or_insert` usage:

</details>

## Using Polonius The Crab for Fun And Profit™

![polonius-the-crab](https://user-images.githubusercontent.com/9920355/165791136-26d09367-3d84-4d09-8f6a-6a3dd91ffc50.jpg)

```rust
#![forbid(unsafe_code)]
use ::polonius_the_crab::prelude::*;
use ::std::collections::HashMap;

/// Typical example of lack-of-Polonius limitation: get_or_insert pattern.
/// See https://nikomatsakis.github.io/rust-belt-rust-2019/#72
fn get_or_insert(
    mut map: &mut HashMap<u32, String>,
) -> &String {
    // Who needs the entry API?
    polonius!(|map| -> &'polonius String {
        if let Some(v) = map.get(&22) {
            polonius_return!(v);
        }
    });
    map.insert(22, String::from("hi"));
    &map[&22]
}
```
