use never_say_never::Never as ǃ;

pub
trait Try<Residual> : Sized {
    type Output;

    fn branch(this: Self)
      -> Result<Self::Output, Residual>
    ;
}

// Trait defined in this direction to hopefully minimize type inference errors.
pub
trait Residual {
    type WithOutput<T>;

    fn with_output<T>(this: Self)
      -> Self::WithOutput<T>
    ;
}

impl<Ok, Err, E> Try<Result<ǃ, Err>> for Result<Ok, E>
where
    Err : From<E>,
{
    type Output = Ok;

    #[inline]
    fn branch(this: Result<Ok, E>)
      -> Result<Ok, Result<ǃ, Err>>
    {
        this.map_err(|e| Err(e.into()))
    }
}

impl<Err> Residual for Result<ǃ, Err> {
    type WithOutput<Ok> = Result<Ok, Err>;

    #[inline]
    fn with_output<Ok>(can_it_be_this_simple: Result<ǃ, Err>)
      -> Result<Ok, Err>
    {
        can_it_be_this_simple?
    }
}

type None = Option<ǃ>;

impl<T> Try<None> for Option<T> {
    type Output = T;

    #[inline]
    fn branch(this: Option<T>)
      -> Result<T, None>
    {
        this.ok_or(None)
    }
}

impl Residual for None {
    type WithOutput<T> = Option<T>;

    #[inline]
    fn with_output<T>(can_it_be_this_simple: None)
      -> Option<T>
    {
        can_it_be_this_simple?
    }
}

/// On 1.67.0 we get a weird interaction with the `WithOutput<T>` GAT when this
/// `Never` definition comes from an external crate. So we re-inline
/// `never_say_never`'s logic here.
mod never_say_never {
    extern crate _never_say_never;
    pub trait FnPtr { type Ret; }
    impl<R> FnPtr for fn() -> R { type Ret = R; }
    pub type Never = <fn() -> ! as FnPtr>::Ret;
}
