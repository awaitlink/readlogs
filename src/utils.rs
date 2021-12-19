#[macro_export]
macro_rules! impl_from_str {
    ($fn:path => $ty:ty) => {
        impl ::std::str::FromStr for $ty {
            type Err = ::anyhow::Error;

            fn from_str(input: &str) -> ::std::result::Result<Self, Self::Err> {
                let (remainder, output) = $fn(input).map_err(|error| {
                    ::anyhow::anyhow!("could not parse `{}` using fn `{}`: {:#?}", stringify!($ty), stringify!($fn), error)
                })?;

                ::anyhow::ensure!(
                    remainder.is_empty(),
                    "could not parse entire input:\n\nRemainder: {:#?}\n\nOutput: {:#?}\n\nInput: {:#?}",
                    remainder,
                    output,
                    input
                );

                ::std::result::Result::Ok(output)
            }
        }
    };
}

pub fn parsing_test<'a, T: std::fmt::Debug>(
    f: impl FnOnce(&'a str) -> nom::IResult<&'a str, T>,
    input: &'a str,
) -> T {
    let (remainder, result) = f(input).unwrap();

    if !remainder.is_empty() {
        panic!(
            "incomplete parsing, got: {:#?}, remainder: {:?}",
            result, remainder
        );
    }

    result
}
