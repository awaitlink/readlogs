use crate::parsers::Span;

#[macro_export]
macro_rules! impl_from_str {
    ($fn:path => $ty:ty) => {
        impl ::std::str::FromStr for $ty {
            type Err = ::anyhow::Error;

            fn from_str(input: &str) -> ::std::result::Result<Self, Self::Err> {
                let (remainder, output) = $fn($crate::utils::span(input)).map_err(|error| {
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

#[macro_export]
macro_rules! traceable_configurable_parser {
    ($vis:vis fn $fn_name:ident < $lifetime:lifetime > ( $($arg_name:ident: $arg_ty:ty),+ $(,)? ) -> $fn_parse_result_ty:ty : | $input_ident:ident | $block:block) => {
        $vis fn $fn_name< $lifetime >($($arg_name: $arg_ty),+) -> impl FnMut(Span< $lifetime >) -> IResult<Span< $lifetime >, $fn_parse_result_ty> {
            move |input| {
                #[cfg(feature = "trace")]
                let name = stringify!($fn_name).to_string() + "("
                    $(+ &format!("{:?}, ", $arg_name))+
                    + ")";

                #[cfg(feature = "trace")]
                let (depth, input) = ::nom_tracable::forward_trace(input, &name);

                let body_ret = {
                    let body = |$input_ident| $block;
                    body(input)
                };

                #[cfg(feature = "trace")]
                return ::nom_tracable::backward_trace(body_ret, &name, depth);

                #[cfg(not(feature = "trace"))]
                return body_ret;
            }
        }
    };
}

pub fn span(input: &str) -> Span {
    Span::new_extra(input, crate::parsers::TraceableInfo::new().parser_width(80))
}

#[cfg(test)]
pub fn test_parsing<'a, T, F>(function: F, input: &'a str, remainder: &'a str, output: T)
where
    T: std::fmt::Debug + PartialEq,
    F: FnOnce(Span<'a>) -> nom::IResult<Span<'a>, T>,
{
    let result = function(span(input)).map(|(remainder, result)| (*remainder.fragment(), result));

    nom_tracable::histogram();
    nom_tracable::cumulative_histogram();

    pretty_assertions::assert_eq!(result, Ok((remainder, output)));
}

#[cfg(test)]
pub fn test_parsing_err_or_remainder<'a, T, F>(function: F, input: &'a str)
where
    T: std::fmt::Debug + PartialEq,
    F: FnOnce(Span<'a>) -> nom::IResult<Span<'a>, T>,
{
    let result = function(span(input));

    nom_tracable::histogram();
    nom_tracable::cumulative_histogram();

    let outcome = match dbg!(result) {
        Ok((remainder, _)) => !remainder.is_empty(),
        Err(_) => true,
    };

    assert!(outcome);
}
