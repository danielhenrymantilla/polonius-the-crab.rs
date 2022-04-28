# The following snippets fail to compile

### `self` cannot be used

```rust ,compile_fail
use ::polonius_the_crab::*;

enum Foo {}
impl Foo {
    fn example (self: &'_ mut Self)
    {
        polonius!(|self| -> () {});
    }
}
```

### The binding must be `mut`

```rust ,compile_fail
use ::polonius_the_crab::*;

fn example ()
{
    let it = &mut ();
    polonius!(|it| -> () {});
}
```

<!-- Templated by `cargo-generate` using https://github.com/danielhenrymantilla/proc-macro-template -->
