use chrono::prelude::*;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{multispace0, newline, space0},
    combinator::{map, opt, verify},
    multi::many0,
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    IResult,
};

use crate::parsers::*;

const LOGS_SECTION_NAME: &str = "Logs";

fn info_section(input: &str) -> IResult<&str, Section<InfoEntry>> {
    let (remainder, name) = verify(
        terminated(common::section_header, opt(newline)),
        |name: &str| name != LOGS_SECTION_NAME,
    )(input)?;

    let (remainder, entries) = many0(delimited(
        multispace0,
        common::key_maybe_enabled_value,
        multispace0,
    ))(remainder)?;

    Ok((
        remainder,
        Section {
            name: name.to_owned(),
            content: entries,
            subsections: vec![],
        },
    ))
}

fn level(input: &str) -> IResult<&str, LogLevel> {
    let (remainder, s) = alt((
        tag(LogLevel::Trace.to_string().to_uppercase().as_str()),
        tag(LogLevel::Debug.to_string().to_uppercase().as_str()),
        tag(LogLevel::Info.to_string().to_uppercase().as_str()),
        tag(LogLevel::Warn.to_string().to_uppercase().as_str()),
        tag(LogLevel::Error.to_string().to_uppercase().as_str()),
        tag(LogLevel::Fatal.to_string().to_uppercase().as_str()),
    ))(input)?;

    Ok((remainder, s.parse().unwrap()))
}

fn metadata(input: &str) -> IResult<&str, (LogLevel, DateTime<Utc>)> {
    separated_pair(
        level,
        space0,
        map(
            common::naive_date_time(None, "-", "T", ":", Some("."), Some("Z")),
            |dt| DateTime::<Utc>::from_utc(dt, Utc),
        ),
    )(input)
}

fn log_entry(input: &str) -> IResult<&str, LogEntry> {
    map(
        tuple((metadata, space0, common::message(metadata))),
        |((lvl, dt), _, message)| LogEntry {
            timestamp: dt.to_string(),
            level: Some(lvl),
            meta: PlatformMetadata::Desktop,
            message,
        },
    )(input)
}

pub fn content(input: &str) -> IResult<&str, Content> {
    let (remainder, (information, logs)) = separated_pair(
        preceded(multispace0, many0(info_section)),
        verify(common::section_header, |name: &str| {
            name == LOGS_SECTION_NAME
        }),
        preceded(multispace0, many0(log_entry)),
    )(input)?;

    Ok((
        remainder,
        Content {
            information,
            logs: vec![Section {
                name: LOGS_SECTION_NAME.to_owned(),
                content: logs,
                subsections: vec![],
            }],
        },
    ))
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use crate::parsing_test;

    use super::*;

    fn test_bucket(country_code: &str, value: u32) -> Bucket {
        Bucket {
            country_code: country_code.to_owned(),
            value: value.to_string(),
        }
    }

    #[test_case("INFO  1234-01-23T12:34:56.789Z" => (LogLevel::Info, Utc.ymd(1234, 1, 23).and_hms_milli(12, 34, 56, 789)); "basic")]
    fn metadata_ok(input: &str) -> (LogLevel, DateTime<Utc>) {
        parsing_test(metadata, input)
    }

    #[test]
    fn content_ok() {
        let (remainder, result) = content("========= Section 1 =========\nKey: 123.456 value\nAnother key: disabled\n\n========= Section 2 =========\nbucketed: enabled 1:2,3:4,*:5\n\n\n\n\n========= Section 3 =========\nabc: disabled true\n\n========= Logs =========\nINFO  1234-01-23T12:34:56.789Z This is a test message.\nDEBUG  1234-01-23T12:34:56.987Z Another message.").unwrap();

        assert_eq!(remainder, "", "remainder should be empty");
        assert_eq!(
            result,
            Content {
                information: vec![
                    Section {
                        name: "Section 1".to_owned(),
                        content: vec![
                            InfoEntry::KeyValue(
                                "Key".to_owned(),
                                Value::Generic("123.456 value".to_owned())
                            ),
                            InfoEntry::KeyEnabledValue("Another key".to_owned(), false, None,),
                        ],
                        subsections: vec![],
                    },
                    Section {
                        name: "Section 2".to_owned(),
                        content: vec![InfoEntry::KeyEnabledValue(
                            "bucketed".to_owned(),
                            true,
                            Some(Value::BucketedFlag(vec![
                                test_bucket("1", 2),
                                test_bucket("3", 4),
                                test_bucket("*", 5),
                            ])),
                        )],
                        subsections: vec![],
                    },
                    Section {
                        name: "Section 3".to_owned(),
                        content: vec![InfoEntry::KeyEnabledValue(
                            "abc".to_owned(),
                            false,
                            Some(Value::Generic("true".to_owned())),
                        )],
                        subsections: vec![],
                    },
                ],
                logs: vec![Section {
                    name: LOGS_SECTION_NAME.to_owned(),
                    content: vec![
                        LogEntry {
                            timestamp: Utc
                                .ymd(1234, 1, 23)
                                .and_hms_milli(12, 34, 56, 789)
                                .to_string(),
                            level: Some(LogLevel::Info),
                            meta: PlatformMetadata::Desktop,
                            message: "This is a test message.".to_owned(),
                        },
                        LogEntry {
                            timestamp: Utc
                                .ymd(1234, 1, 23)
                                .and_hms_milli(12, 34, 56, 987)
                                .to_string(),
                            level: Some(LogLevel::Debug),
                            meta: PlatformMetadata::Desktop,
                            message: "Another message.".to_owned(),
                        }
                    ],
                    subsections: vec![],
                }],
            }
        );
    }
}
