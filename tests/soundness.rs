use ::core::ops::Not;

#[test]
fn cargo_check_with_polonius ()
{
    ::std::process::Command::new(env!("CARGO"))
        .env("RUSTC_BOOTSTRAP", "1")
        .args([
            "rustc", "--profile=check",
            "--features", "polonius",
            "--quiet",
            "--", "-Zpolonius", "-Funsafe_code",
        ])
        .status()
        .unwrap()
        .success()
        .not()
        .then(|| panic!())
    ;
}
