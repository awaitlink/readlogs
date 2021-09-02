#[macro_export]
macro_rules! fetch_fn {
    ($fn:ident, $msg:ident($result:ty)) => {
        fn $fn(&self, url: &str) -> ::anyhow::Result<::yew::services::fetch::FetchTask> {
            use ::yew::{
                format::Nothing,
                services::fetch::{FetchService, Request, Response},
            };

            let request = Request::get(url).body(Nothing)?;
            let callback =
                self.link
                    .callback(move |response: Response<::anyhow::Result<$result>>| {
                        let (meta, result) = response.into_parts();

                        crate::Msg::$msg(if meta.status.is_success() {
                            result
                        } else {
                            Err(::anyhow::anyhow!(
                                "{}: !meta.status.is_success(): {:#?}",
                                stringify!($fn),
                                meta
                            ))
                        })
                    });

            FetchService::$fn(request, callback)
        }
    };
}

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

pub fn parsing_test<'a, T>(
    f: impl FnOnce(&'a str) -> nom::IResult<&'a str, T>,
    input: &'a str,
) -> T {
    let (remainder, result) = f(input).unwrap();
    assert_eq!(remainder, "", "remainder should be empty");
    result
}
