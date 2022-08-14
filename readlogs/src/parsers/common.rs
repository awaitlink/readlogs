use chrono::prelude::*;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_until},
    character::complete::{digit1, multispace0, newline, space0},
    combinator::{eof, map, not, opt, peek, value, verify},
    error::ParseError,
    multi::{many1, many_till, separated_list1},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    IResult,
};

use crate::parsers::*;

// https://docs.rs/nom/6.2.1/nom/recipes/index.html#whitespace
pub fn ws<'a, F: 'a, O, E>(inner: F) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, O, E>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O, E>,
    E: ParseError<Span<'a>>,
{
    delimited(space0, inner, space0)
}

pub fn multispaced0<'a, F: 'a, O, E>(inner: F) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, O, E>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O, E>,
    E: ParseError<Span<'a>>,
{
    delimited(multispace0, inner, multispace0)
}

/// Parses a section header and outputs its name.
#[traceable_parser]
pub fn section_header(input: Span) -> IResult<Span, &str> {
    let section_decoration = |input| many1(tag("="))(input);
    let section_name = map(take_until("="), |words: Span| words.fragment().trim());

    delimited(section_decoration, ws(section_name), section_decoration)(input)
}

#[traceable_parser]
fn bucket(input: Span) -> IResult<Span, Bucket> {
    map(
        separated_pair(is_not(":\n"), tag(":"), digit1),
        |(country_code, value): (Span, Span)| Bucket {
            country_code: country_code.fragment().to_string(),
            value: value.fragment().to_string(),
        },
    )(input)
}

#[traceable_parser]
pub fn bucketed_flag(input: Span) -> IResult<Span, Vec<Bucket>> {
    terminated(
        separated_list1(tag(","), bucket),
        peek(alt((tag("\n"), eof))),
    )(input)
}

#[traceable_configurable_parser]
pub fn key_maybe_enabled_value<'a>(
    inside_inline_section: bool,
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, InfoEntry> {
    let parse_key = terminated(
        verify(take_until(": "), |key: &Span| !key.contains('\n')),
        tag(": "),
    );

    let parse_enabled = alt((value(true, tag("enabled")), value(false, tag("disabled"))));

    let parse_value = alt((
        map(bucketed_flag, Value::BucketedFlag),
        map(
            is_not(if inside_inline_section { "\n|" } else { "\n" }),
            |s: Span| Value::Generic(s.trim().to_owned()),
        ),
    ));

    map(
        tuple((
            peek(not(alt((
                value((), tag("--")),
                value((), android::jobs_inline_section),
            )))),
            parse_key,
            opt(parse_enabled),
            delimited(
                space0,
                opt(parse_value),
                peek(alt((tag("\n"), tag("|"), eof))),
            ),
        )),
        |(_, k, enabled, v)| match enabled {
            Some(enabled) => InfoEntry::KeyEnabledValue(k.trim().to_owned(), enabled, v),
            None => InfoEntry::KeyValue(k.trim().to_owned(), v.unwrap()),
        },
    )(input)
}

#[traceable_configurable_parser]
pub fn naive_date_time<'a>(
    assumed_year: Option<i32>,
    ymd_separator: &'a str,
    ymd_hms_separator: &'a str,
    hms_separator: &'a str,
    millisecond_separator: Option<&'a str>,
    ending: Option<&'a str>,
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, NaiveDateTime> {
    let (remainder, year) = match assumed_year {
        Some(year) => (input, year),
        None => map(terminated(digit1, tag(ymd_separator)), |year: Span| {
            year.parse().unwrap()
        })(input)?,
    };

    let (mut remainder, (month, _, day, _, hour, _, minute, _, second)) = tuple((
        digit1,
        tag(ymd_separator),
        digit1,
        tag(ymd_hms_separator),
        digit1,
        tag(hms_separator),
        digit1,
        tag(hms_separator),
        digit1,
    ))(remainder)?;

    let date = NaiveDate::from_ymd(year, month.parse().unwrap(), day.parse().unwrap());

    let datetime = if let Some(millisecond_separator) = millisecond_separator {
        let (new_remainder, millisecond) = preceded(tag(millisecond_separator), digit1)(remainder)?;

        remainder = new_remainder;

        date.and_hms_milli(
            hour.parse().unwrap(),
            minute.parse().unwrap(),
            second.parse().unwrap(),
            millisecond.parse().unwrap(),
        )
    } else {
        date.and_hms(
            hour.parse().unwrap(),
            minute.parse().unwrap(),
            second.parse().unwrap(),
        )
    };

    if let Some(ending) = ending {
        let (new_remainder, _) = tag(ending)(remainder)?;
        remainder = new_remainder;
    }

    Ok((remainder, datetime))
}

/// Parses log message contents until a new `metadata` is encountered or the end of the input.
pub fn message<'a, F: 'a, O, E: ParseError<Span<'a>>>(
    metadata: F,
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, String, E>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O, E>,
{
    map(
        many_till(
            terminated(alt((is_not("\n"), take_until("\n"))), opt(newline)),
            peek(alt((value((), metadata), value((), eof)))),
        ),
        |(strings, _)| {
            strings
                .into_iter()
                .map(|span| *span.fragment())
                .collect::<Vec<_>>()
                .join("\n")
        },
    )
}

#[cfg(test)]
pub use tests::test_bucket;

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;
    use crate::utils::*;

    pub fn test_bucket(country_code: &str, value: u32) -> Bucket {
        Bucket {
            country_code: country_code.to_owned(),
            value: value.to_string(),
        }
    }

    #[test_case("========== Name ==========", "Name")]
    #[test_case("===== Multiple Words =====", "Multiple Words")]
    #[test_case("=============   Odd   ====", "Odd")]
    fn section_header_ok(input: &str, output: &str) {
        test_parsing(section_header, input, "", output);
    }

    #[test_case("1:2", test_bucket("1", 2); "digit")]
    #[test_case("*:1", test_bucket("*", 1); "other")]
    fn bucket_ok(input: &str, output: Bucket) {
        test_parsing(bucket, input, "", output);
    }

    #[test]
    fn bucket_err() {
        test_parsing_err_or_remainder(
            bucket,
            "true\n\n========= Logs =========\nINFO  1234-01-23T12:34:56.789Z Message",
        );
    }

    #[test_case("1:2,3:4,*:5", "", vec![
        test_bucket("1", 2),
        test_bucket("3", 4),
        test_bucket("*", 5),
    ]; "basic")]
    #[test_case("1:2,3:4,*:5\n\n========= Logs =========\nINFO  1234-01-23T12:34:56.789Z Message", "\n\n========= Logs =========\nINFO  1234-01-23T12:34:56.789Z Message", vec![
        test_bucket("1", 2),
        test_bucket("3", 4),
        test_bucket("*", 5),
    ]; "followed by log section")]
    fn bucketed_flag_ok(input: &str, remainder: &str, output: Vec<Bucket>) {
        test_parsing(bucketed_flag, input, remainder, output);
    }

    #[test]
    fn bucketed_flag_err() {
        test_parsing_err_or_remainder(
            bucketed_flag,
            "true\n\n========= Logs =========\nINFO  1234-01-23T12:34:56.789Z Message",
        );
    }

    #[test_case("abc.defGhi.jkl123: 1:2,3:4,*:5", "", InfoEntry::KeyValue(
        "abc.defGhi.jkl123".to_owned(),
        Value::BucketedFlag(vec![
            test_bucket("1", 2),
            test_bucket("3", 4),
            test_bucket("*", 5),
        ]),
    ); "bucketed")]
    #[test_case("abc.defGhi.jkl123: enabled 1:2,3:4,*:5", "", InfoEntry::KeyEnabledValue(
        "abc.defGhi.jkl123".to_owned(),
        true,
        Some(Value::BucketedFlag(vec![
            test_bucket("1", 2),
            test_bucket("3", 4),
            test_bucket("*", 5),
        ])),
    ); "enabled bucketed")]
    #[test_case("abc-defghi-jkl123: generic value", "", InfoEntry::KeyValue(
        "abc-defghi-jkl123".to_owned(),
        Value::Generic("generic value".to_owned()),
    ); "generic value")]
    #[test_case("abc.defGhi.jkl123: disabled \\slash", "", InfoEntry::KeyEnabledValue(
        "abc.defGhi.jkl123".to_owned(),
        false,
        Some(Value::Generic("\\slash".to_owned())),
    ); "disabled generic value")]
    #[test_case("abc.defGhi.jkl123: disabled", "", InfoEntry::KeyEnabledValue(
        "abc.defGhi.jkl123".to_owned(),
        false,
        None,
    ); "disabled but no value")]
    #[test_case("abc.defGhi.jkl123: disabled //../.123//..abc\n", "\n", InfoEntry::KeyEnabledValue(
        "abc.defGhi.jkl123".to_owned(),
        false,
        Some(Value::Generic("//../.123//..abc".to_owned())),
    ); "followed by 1 newline")]
    #[test_case("abc.defGhi.jkl123: 12:34:56\n", "\n", InfoEntry::KeyValue(
        "abc.defGhi.jkl123".to_owned(),
        Value::Generic("12:34:56".to_owned()),
    ); "time in value")]
    #[test_case("abc.defGhi.jkl123: [A, BC, DEF]\n", "\n", InfoEntry::KeyValue(
        "abc.defGhi.jkl123".to_owned(),
        Value::Generic("[A, BC, DEF]".to_owned()),
    ); "array in value")]
    #[test_case(
        "abc: disabled true\n\n========= Logs =========\nINFO  1234-01-23T12:34:56.789Z This is a test message.",
        "\n\n========= Logs =========\nINFO  1234-01-23T12:34:56.789Z This is a test message.",
        InfoEntry::KeyEnabledValue(
            "abc".to_owned(),
            false,
            Some(Value::Generic("true".to_owned())),
        );
        "followed by log section"
    )]
    fn key_maybe_enabled_value_ok(input: &str, remainder: &str, output: InfoEntry) {
        test_parsing(key_maybe_enabled_value(false), input, remainder, output);
    }

    #[test]
    fn key_maybe_enabled_value_err() {
        test_parsing_err_or_remainder(key_maybe_enabled_value(false), "-- test : 123");
    }

    #[test_case("1234/01/23 12:34:56:789", NaiveDate::from_ymd(1234, 1, 23).and_hms_milli(12, 34, 56, 789); "basic")]
    fn timestamp_ok(input: &str, output: NaiveDateTime) {
        test_parsing(
            naive_date_time(None, "/", " ", ":", Some(":"), None),
            input,
            "",
            output,
        );
    }
}
