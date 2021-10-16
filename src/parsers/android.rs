use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, take_until},
    character::complete::{self, digit1, multispace0, newline, space0, space1},
    combinator::{map, not, opt, peek, success, value, verify},
    multi::{count, many0, many1, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, tuple},
    IResult,
};

use crate::{parsers::*, post_processing};

const LOGCAT_SECTION_NAME: &str = "LOGCAT";
const LOGGER_SECTION_NAME: &str = "LOGGER";

#[derive(Debug, Clone, Copy, PartialEq)]
enum SectionLevel {
    Base,
    Sub,
}

fn subsection_header(input: &str) -> IResult<&str, &str> {
    preceded(pair(many1(tag("-")), tag(" ")), is_not("\n"))(input)
}

fn thread(input: &str) -> IResult<&str, InfoEntry> {
    map(
        separated_pair(delimited(tag("["), digit1, tag("]")), space1, is_not("\n")),
        |(n, s): (&str, &str)| InfoEntry::KeyValue(n.to_owned(), Value::Generic(s.to_owned())),
    )(input)
}

fn jobs_inline_section(input: &str) -> IResult<&str, Section<InfoEntry>> {
    map(
        separated_pair(
            preceded(alt((tag("id: "), tag("jobSpecId: "))), is_not(" ")),
            common::ws(tag("|")),
            separated_list1(
                common::ws(tag("|")),
                common::ws(common::key_maybe_enabled_value),
            ),
        ),
        |(name, pairs)| Section {
            name: name.to_owned(),
            content: pairs,
            subsections: vec![],
        },
    )(input)
}

fn local_metrics_subsection(input: &str) -> IResult<&str, Section<InfoEntry>> {
    map(
        tuple((
            is_not("\n"),
            count(
                common::multispaced0(verify(
                    common::key_maybe_enabled_value,
                    |entry| match entry {
                        InfoEntry::KeyValue(k, _) => ["p50", "p90", "p99"].contains(&k.as_str()),
                        _ => false,
                    },
                )),
                3,
            ),
        )),
        |(name, content)| Section {
            name: name.to_owned(),
            content,
            subsections: vec![],
        },
    )(input)
}

fn local_metrics_section(input: &str) -> IResult<&str, Section<InfoEntry>> {
    map(
        tuple((
            is_not("\n"),
            count(
                common::multispaced0(verify(
                    common::key_maybe_enabled_value,
                    |entry| match entry {
                        InfoEntry::KeyValue(k, _) => {
                            ["count", "p50", "p90", "p99"].contains(&k.as_str())
                        }
                        _ => false,
                    },
                )),
                4,
            ),
            many0(local_metrics_subsection),
        )),
        |(name, content, subsections)| Section {
            name: name.to_owned(),
            content,
            subsections,
        },
    )(input)
}

fn info_section(depth: SectionLevel) -> impl FnMut(&str) -> IResult<&str, Section<InfoEntry>> {
    move |input| {
        let section_header_parser = match depth {
            SectionLevel::Base => common::section_header,
            SectionLevel::Sub => subsection_header,
        };

        let (remainder, name) = verify(
            delimited(multispace0, section_header_parser, opt(newline)),
            |name: &str| name != LOGCAT_SECTION_NAME && name != LOGGER_SECTION_NAME,
        )(input)?;

        let (remainder, content) = alt((
            preceded(
                peek(not(jobs_inline_section)),
                common::multispaced0(alt((
                    many1(common::multispaced0(common::key_maybe_enabled_value)),
                    many1(common::multispaced0(thread)),
                    map(remote_object::remote_object, |ro| {
                        vec![InfoEntry::RemoteObject(ro)]
                    }),
                    value(vec![InfoEntry::LiteralNone], tag("None")),
                    many1(common::multispaced0(map(
                        preceded(peek(not(local_metrics_section)), is_not("\n-=")),
                        |s: &str| InfoEntry::Generic(s.to_owned()),
                    ))),
                ))),
            ),
            success(vec![]),
        ))(remainder)?;

        let (remainder, subsections) = match depth {
            SectionLevel::Base => alt((
                many1(info_section(SectionLevel::Sub)),
                many0(common::multispaced0(local_metrics_section)),
            ))(remainder)?,
            SectionLevel::Sub => many0(common::multispaced0(jobs_inline_section))(remainder)?,
        };

        Ok((
            remainder,
            Section {
                name: name.to_owned(),
                content,
                subsections,
            },
        ))
    }
}

fn logcat_entry(year: i32) -> impl FnMut(&str) -> IResult<&str, LogEntry> {
    move |input| {
        map(
            tuple((
                common::naive_date_time(Some(year), "-", " ", ":", Some("."), None),
                space0,
                is_not(" "),
                space0,
                is_not(" "),
                space0,
                is_a("VDIWEF"),
                space0,
                take_until(": "),
                tag(": "),
                space0,
                alt((is_not("\n"), success(""))),
            )),
            |(dt, _, process_id, _, thread_id, _, level, _, tag, _, _, message)| LogEntry {
                timestamp: dt.to_string(),
                level: Some(level.parse().unwrap()),
                meta: PlatformMetadata::AndroidLogcat(
                    process_id.to_owned(),
                    thread_id.to_owned(),
                    tag.trim().to_owned(),
                ),
                message: message.to_owned(),
            },
        )(input)
    }
}

fn logcat_section(year: i32) -> impl FnMut(&str) -> IResult<&str, Section<LogEntry>> {
    move |input| {
        preceded(
            common::multispaced0(verify(common::section_header, |name: &str| {
                name == LOGCAT_SECTION_NAME
            })),
            map(
                many0(map(
                    pair(
                        common::multispaced0(subsection_header),
                        many0(common::multispaced0(logcat_entry(year))),
                    ),
                    |(name, content)| Section {
                        name: name.to_owned(),
                        content: post_processing::collapse_log_entries(content),
                        subsections: vec![],
                    },
                )),
                |subsections| Section {
                    name: LOGCAT_SECTION_NAME.to_owned(),
                    content: vec![],
                    subsections,
                },
            ),
        )(input)
    }
}

fn logger_metadata(input: &str) -> IResult<&str, (PlatformMetadata, String, LogLevel)> {
    enum LoggerTimezone<'a> {
        Parsed(FixedOffset),
        Unparsed(&'a str),
    }

    let logger_timezone = alt((
        map(
            tuple((
                tag("GMT"),
                alt((value(1, tag("+")), value(-1, tag("-")))),
                complete::i32,
                tag(":"),
                complete::i32,
            )),
            |(_, pm, h, _, m)| LoggerTimezone::Parsed(FixedOffset::east((h * 60 + m) * 60 * pm)),
        ),
        map(take_until(" "), LoggerTimezone::Unparsed),
    ));

    map(
        tuple((
            delimited(tag("["), is_not("]"), tag("]")),
            space0,
            delimited(tag("["), is_not("]"), tag("]")),
            space0,
            common::naive_date_time(None, "-", " ", ":", Some("."), None),
            space0,
            logger_timezone,
            space0,
            is_not(" "),
            space0,
            take_until(": "),
            tag(": "),
        )),
        |(version, _, thread_id, _, dt, _, tz, _, level, _, tag, _)| {
            (
                PlatformMetadata::AndroidLogger(
                    version.to_owned(),
                    thread_id.trim().to_owned(),
                    tag.trim().to_owned(),
                ),
                match tz {
                    LoggerTimezone::Parsed(tz) => tz.from_local_datetime(&dt).unwrap().to_string(),
                    LoggerTimezone::Unparsed(s) => dt.to_string() + " " + s,
                },
                level.parse().unwrap(),
            )
        },
    )(input)
}

fn logger_entry(input: &str) -> IResult<&str, LogEntry> {
    map(
        separated_pair(logger_metadata, space0, common::message(logger_metadata)),
        |((meta, timestamp, level), message)| LogEntry {
            timestamp,
            level: Some(level),
            meta,
            message,
        },
    )(input)
}

pub fn content(input: &str) -> IResult<&str, Content> {
    let (remainder, (information, logcat_section, _, mut logger_entries)) = tuple((
        preceded(multispace0, many0(info_section(SectionLevel::Base))),
        preceded(multispace0, logcat_section(Utc::today().year())), // TODO: year...
        verify(common::section_header, |name: &str| {
            name == LOGGER_SECTION_NAME
        }),
        preceded(multispace0, many0(common::multispaced0(logger_entry))),
    ))(input)?;

    logger_entries = post_processing::collapse_log_entries(logger_entries);

    Ok((
        remainder,
        Content {
            information,
            logs: vec![
                logcat_section,
                Section {
                    name: LOGGER_SECTION_NAME.to_owned(),
                    content: logger_entries,
                    subsections: vec![],
                },
            ],
        },
    ))
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;
    use crate::parsing_test;

    #[test_case("-- Abc" => "Abc"; "basic")]
    #[test_case("--------- Long line" => "Long line"; "long")]
    fn subsection_header_ok(input: &str) -> &str {
        parsing_test(subsection_header, input)
    }

    #[test_case(
        "id: JOB::abcd1234-efgh-5678-ijkl-9012mnop1234 | a: TestJob | b: _test_value_ | number: 123 | negative: -1"
        => Section {
            name: "JOB::abcd1234-efgh-5678-ijkl-9012mnop1234".to_owned(),
            content: vec![
                InfoEntry::KeyValue("a".to_owned(), Value::Generic("TestJob".to_owned())),
                InfoEntry::KeyValue("b".to_owned(), Value::Generic("_test_value_".to_owned())),
                InfoEntry::KeyValue("number".to_owned(), Value::Generic("123".to_owned())),
                InfoEntry::KeyValue("negative".to_owned(), Value::Generic("-1".to_owned())),
            ],
            subsections: vec![],
        };
        "job"
    )]
    #[test_case(
        "jobSpecId: JOB::abcd1234-efgh-5678-ijkl-9012mnop1234 | a: TestConstraint | anotherValue: false"
        => Section {
            name: "JOB::abcd1234-efgh-5678-ijkl-9012mnop1234".to_owned(),
            content: vec![
                InfoEntry::KeyValue("a".to_owned(), Value::Generic("TestConstraint".to_owned())),
                InfoEntry::KeyValue("anotherValue".to_owned(), Value::Generic("false".to_owned())),
            ],
            subsections: vec![],
        };
        "constraint"
    )]
    fn jobs_inline_section_ok(input: &str) -> Section<InfoEntry> {
        parsing_test(jobs_inline_section, input)
    }

    #[test_case(
        "========= HEADER =========\nTime          : 1234567890123\nDays Installed: 123\nself.isRegistered()  : true" =>
        Section {
            name: "HEADER".to_owned(),
            content: vec![
                InfoEntry::KeyValue("Time".to_owned(), Value::Generic("1234567890123".to_owned())),
                InfoEntry::KeyValue("Days Installed".to_owned(), Value::Generic("123".to_owned())),
                InfoEntry::KeyValue("self.isRegistered()".to_owned(), Value::Generic("true".to_owned())),
            ],
            subsections: vec![],
        }; "sysinfo, constraints, key preferences, permissions"
    )]
    #[test_case(
        "========== JOBS ===========\n-- Jobs\nid: JOB::abcd1234-efgh-5678-ijkl-9012mnop1234 | a: TestJob | b: _test_value_ | number: 123 | negative: -1\n\n-- Constraints\njobSpecId: JOB::abcd1234-efgh-5678-ijkl-9012mnop1234 | a: TestConstraint | anotherValue: false\n\n-- Dependencies\nNone" =>
        Section {
            name: "JOBS".to_owned(),
            content: vec![],
            subsections: vec![
                Section {
                    name: "Jobs".to_owned(),
                    content: vec![],
                    subsections: vec![Section {
                        name: "JOB::abcd1234-efgh-5678-ijkl-9012mnop1234".to_owned(),
                        content: vec![
                            InfoEntry::KeyValue("a".to_owned(), Value::Generic("TestJob".to_owned())),
                            InfoEntry::KeyValue("b".to_owned(), Value::Generic("_test_value_".to_owned())),
                            InfoEntry::KeyValue("number".to_owned(), Value::Generic("123".to_owned())),
                            InfoEntry::KeyValue("negative".to_owned(), Value::Generic("-1".to_owned())),
                        ],
                        subsections: vec![],
                    }],
                },
                Section {
                    name: "Constraints".to_owned(),
                    content: vec![],
                    subsections: vec![Section {
                        name: "JOB::abcd1234-efgh-5678-ijkl-9012mnop1234".to_owned(),
                        content: vec![
                            InfoEntry::KeyValue("a".to_owned(), Value::Generic("TestConstraint".to_owned())),
                            InfoEntry::KeyValue("anotherValue".to_owned(), Value::Generic("false".to_owned())),
                        ],
                        subsections: vec![],
                    }],
                },
                Section {
                    name: "Dependencies".to_owned(),
                    content: vec![InfoEntry::LiteralNone],
                    subsections: vec![],
                },
            ],
        }; "jobs"
    )]
    #[test_case(
        "====== HEADER =======\n-- Abc\nABC123          : true\nCapability Name : false\n\n-- Def\nABC123          : SUPPORTED\nCapability Name : NOT_SUPPORTED\nexample.testFlag: 1:2,3:4,*:5" =>
        Section {
            name: "HEADER".to_owned(),
            content: vec![],
            subsections: vec![
                Section {
                    name: "Abc".to_owned(),
                    content: vec![
                        InfoEntry::KeyValue("ABC123".to_owned(), Value::Generic("true".to_owned())),
                        InfoEntry::KeyValue("Capability Name".to_owned(), Value::Generic("false".to_owned())),
                    ],
                    subsections: vec![],
                },
                Section {
                    name: "Def".to_owned(),
                    content: vec![
                        InfoEntry::KeyValue("ABC123".to_owned(), Value::Generic("SUPPORTED".to_owned())),
                        InfoEntry::KeyValue("Capability Name".to_owned(), Value::Generic("NOT_SUPPORTED".to_owned())),
                        InfoEntry::KeyValue("example.testFlag".to_owned(), Value::BucketedFlag(vec![
                            common::test_bucket("1", 2),
                            common::test_bucket("3", 4),
                            common::test_bucket("*", 5),
                        ])),
                    ],
                    subsections: vec![],
                },
            ],
        }; "capabilities, feature flags"
    )]
    #[test_case(
        "====== LOCAL METRICS ======\ncold-start-conversation-list\n  count: 5\n  p50: 3456\n  p90: 4567\n  p99: 4567\n    application-create\n      p50: 123\n      p90: 456\n      p99: 456\n    data-loaded\n      p50: 456\n      p90: 789\n      p99: 789\n\n\nconversation-open\n  count: 123\n  p50: 1234\n  p90: 5678\n  p99: 12345" =>
        Section {
            name: "LOCAL METRICS".to_owned(),
            content: vec![],
            subsections: vec![
                Section {
                    name: "cold-start-conversation-list".to_owned(),
                    content: vec![
                        InfoEntry::KeyValue("count".to_owned(), Value::Generic("5".to_owned())),
                        InfoEntry::KeyValue("p50".to_owned(), Value::Generic("3456".to_owned())),
                        InfoEntry::KeyValue("p90".to_owned(), Value::Generic("4567".to_owned())),
                        InfoEntry::KeyValue("p99".to_owned(), Value::Generic("4567".to_owned())),
                    ],
                    subsections: vec![
                        Section {
                            name: "application-create".to_owned(),
                            content: vec![
                                InfoEntry::KeyValue("p50".to_owned(), Value::Generic("123".to_owned())),
                                InfoEntry::KeyValue("p90".to_owned(), Value::Generic("456".to_owned())),
                                InfoEntry::KeyValue("p99".to_owned(), Value::Generic("456".to_owned())),
                            ],
                            subsections: vec![],
                        },
                        Section {
                            name: "data-loaded".to_owned(),
                            content: vec![
                                InfoEntry::KeyValue("p50".to_owned(), Value::Generic("456".to_owned())),
                                InfoEntry::KeyValue("p90".to_owned(), Value::Generic("789".to_owned())),
                                InfoEntry::KeyValue("p99".to_owned(), Value::Generic("789".to_owned())),
                            ],
                            subsections: vec![],
                        },
                    ],
                },
                Section {
                    name: "conversation-open".to_owned(),
                    content: vec![
                        InfoEntry::KeyValue("count".to_owned(), Value::Generic("123".to_owned())),
                        InfoEntry::KeyValue("p50".to_owned(), Value::Generic("1234".to_owned())),
                        InfoEntry::KeyValue("p90".to_owned(), Value::Generic("5678".to_owned())),
                        InfoEntry::KeyValue("p99".to_owned(), Value::Generic("12345".to_owned())),
                    ],
                    subsections: vec![],
                },
            ],
        }; "local metrics"
    )]
    #[test_case(
        "======== PIN STATE ========\nKey: abc_def_ghi\nTest Value: 1234567890\nAbcDef: true" =>
        Section {
            name: "PIN STATE".to_owned(),
            content: vec![
                InfoEntry::KeyValue("Key".to_owned(), Value::Generic("abc_def_ghi".to_owned())),
                InfoEntry::KeyValue("Test Value".to_owned(), Value::Generic("1234567890".to_owned())),
                InfoEntry::KeyValue("AbcDef".to_owned(), Value::Generic("true".to_owned())),
            ],
            subsections: vec![],
        }; "pin state (does not have space-alignment)"
    )]
    #[test_case(
        "========== POWER ==========\nCurrent bucket: Frequent\nHighest bucket: Active\nLowest bucket : Rare\n\nMon Jan 23 12:34:56 GMT+01:00 1234: Bucket Change: Active\nMon Jan 23 12:34:57 GMT+01:00 1234: Bucket Change: Rare\nMon Jan 23 12:34:58 GMT+01:00 1234: Bucket Change: Frequent" =>
        Section {
            name: "POWER".to_owned(),
            content: vec![
                InfoEntry::KeyValue("Current bucket".to_owned(), Value::Generic("Frequent".to_owned())),
                InfoEntry::KeyValue("Highest bucket".to_owned(), Value::Generic("Active".to_owned())),
                InfoEntry::KeyValue("Lowest bucket".to_owned(), Value::Generic("Rare".to_owned())),
                InfoEntry::KeyValue("Mon Jan 23 12:34:56 GMT+01:00 1234".to_owned(), Value::Generic("Bucket Change: Active".to_owned())),
                InfoEntry::KeyValue("Mon Jan 23 12:34:57 GMT+01:00 1234".to_owned(), Value::Generic("Bucket Change: Rare".to_owned())),
                InfoEntry::KeyValue("Mon Jan 23 12:34:58 GMT+01:00 1234".to_owned(), Value::Generic("Bucket Change: Frequent".to_owned())),
            ],
            subsections: vec![],
        }; "power"
    )]
    #[test_case(
        "====== NOTIFICATIONS ======\nTest key        : true\nAnother test key: false\n\n-- abc_def_v2\ntest       : LOW (2)\nanotherTest: N/A (Requires API 30)" =>
        Section {
            name: "NOTIFICATIONS".to_owned(),
            content: vec![
                InfoEntry::KeyValue("Test key".to_owned(), Value::Generic("true".to_owned())),
                InfoEntry::KeyValue("Another test key".to_owned(), Value::Generic("false".to_owned())),
            ],
            subsections: vec![Section {
                name: "abc_def_v2".to_owned(),
                content: vec![
                    InfoEntry::KeyValue("test".to_owned(), Value::Generic("LOW (2)".to_owned())),
                    InfoEntry::KeyValue("anotherTest".to_owned(), Value::Generic("N/A (Requires API 30)".to_owned())),
                ],
                subsections: vec![],
            }],
        }; "notifications"
    )]
    #[test_case(
        "========== TRACE ==========\nhttps://debuglogs.org/0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123" =>
        Section {
            name: "TRACE".to_owned(),
            content: vec![InfoEntry::RemoteObject(
                RemoteObject::new_unchecked(Platform::Android, "0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123")
            )],
            subsections: vec![],
        }; "trace"
    )]
    #[test_case(
        "========= THREADS =========\n[1] main\n[1234] Signal Catcher\n[1235] AbcDefGhi\n[6789] OkHttp https://abc-def.example.org/..." =>
        Section {
            name: "THREADS".to_owned(),
            content: vec![
                InfoEntry::KeyValue("1".to_owned(), Value::Generic("main".to_owned())),
                InfoEntry::KeyValue("1234".to_owned(), Value::Generic("Signal Catcher".to_owned())),
                InfoEntry::KeyValue("1235".to_owned(), Value::Generic("AbcDefGhi".to_owned())),
                InfoEntry::KeyValue("6789".to_owned(), Value::Generic("OkHttp https://abc-def.example.org/...".to_owned())),
            ],
            subsections: vec![],
        }; "threads"
    )]
    #[test_case(
        "===== BLOCKED THREADS =====\n-- [9876] AbcDefGhi (BLOCKED)\nghi.jkl.ABCdef.abcDefGhi(Native Method)\nabc.def.Abc$Cba.run(DEF.java:456)\nabc.def.Def.run(ABC.java:123)" =>
        Section {
            name: "BLOCKED THREADS".to_owned(),
            content: vec![],
            subsections: vec![
                Section {
                    name: "[9876] AbcDefGhi (BLOCKED)".to_owned(),
                    content: vec![
                        InfoEntry::Generic("ghi.jkl.ABCdef.abcDefGhi(Native Method)".to_owned()),
                        InfoEntry::Generic("abc.def.Abc$Cba.run(DEF.java:456)".to_owned()),
                        InfoEntry::Generic("abc.def.Def.run(ABC.java:123)".to_owned()),
                    ],
                    subsections: vec![],
                }
            ],
        }; "blocked threads"
    )]
    #[test_case(
        "\n\n \n====== EMPTY SECTION ======\n" =>
        Section {
            name: "EMPTY SECTION".to_owned(),
            content: vec![],
            subsections: vec![],
        }; "empty section"
    )]
    fn info_section_ok(input: &str) -> Section<InfoEntry> {
        parsing_test(info_section(SectionLevel::Base), input)
    }

    #[test_case("01-23 12:34:56.789 12345 12367 I abc: Log message" => LogEntry {
        timestamp: NaiveDate::from_ymd(1234, 1, 23).and_hms_milli(12, 34, 56, 789).to_string(),
        level: Some(LogLevel::Info),
        meta: PlatformMetadata::AndroidLogcat("12345".to_owned(), "12367".to_owned(), "abc".to_owned()),
        message: "Log message".to_owned(),
    }; "basic")]
    #[test_case("01-23 12:34:56.789 12345 12367 I abc: " => LogEntry {
        timestamp: NaiveDate::from_ymd(1234, 1, 23).and_hms_milli(12, 34, 56, 789).to_string(),
        level: Some(LogLevel::Info),
        meta: PlatformMetadata::AndroidLogcat("12345".to_owned(), "12367".to_owned(), "abc".to_owned()),
        message: "".to_owned(),
    }; "no message")]
    fn logcat_entry_ok(input: &str) -> LogEntry {
        parsing_test(logcat_entry(1234), input)
    }

    #[test_case("========= LOGCAT ==========\n--------- beginning of crash\n01-21 12:34:56.789  1234  5678 F libc    : Fatal signal 11 (SIGSEGV), code 2, fault addr 0x12345678 in tid 9876 (Abc)\n--------- beginning of main\n01-22 12:34:56.789 12345 12367 I chatty  : uid=10001(org.thoughtcrime.securesms) expire 1 line\n01-23 12:34:56.789 12345 12367 I chatty  : uid=10001(org.thoughtcrime.securesms) expire 5 lines"
    => Section {
        name: LOGCAT_SECTION_NAME.to_owned(),
        content: vec![],
        subsections: vec![
            Section {
                name: "beginning of crash".to_owned(),
                content: vec![
                    LogEntry {
                        timestamp: NaiveDate::from_ymd(1234, 1, 21).and_hms_milli(12, 34, 56, 789).to_string(),
                        level: Some(LogLevel::Fatal),
                        meta: PlatformMetadata::AndroidLogcat("1234".to_owned(), "5678".to_owned(), "libc".to_owned()),
                        message: "Fatal signal 11 (SIGSEGV), code 2, fault addr 0x12345678 in tid 9876 (Abc)".to_owned(),
                    }
                ],
                subsections: vec![],
            },
            Section {
                name: "beginning of main".to_owned(),
                content: vec![
                    LogEntry {
                        timestamp: NaiveDate::from_ymd(1234, 1, 22).and_hms_milli(12, 34, 56, 789).to_string(),
                        level: Some(LogLevel::Info),
                        meta: PlatformMetadata::AndroidLogcat("12345".to_owned(), "12367".to_owned(), "chatty".to_owned()),
                        message: "uid=10001(org.thoughtcrime.securesms) expire 1 line".to_owned(),
                    },
                    LogEntry {
                        timestamp: NaiveDate::from_ymd(1234, 1, 23).and_hms_milli(12, 34, 56, 789).to_string(),
                        level: Some(LogLevel::Info),
                        meta: PlatformMetadata::AndroidLogcat("12345".to_owned(), "12367".to_owned(), "chatty".to_owned()),
                        message: "uid=10001(org.thoughtcrime.securesms) expire 5 lines".to_owned(),
                    },
                ],
                subsections: vec![],
            },
        ],
    }; "basic")]
    fn logcat_section_ok(input: &str) -> Section<LogEntry> {
        parsing_test(logcat_section(1234), input)
    }

    #[test_case("[1.23.4] [5678 ] 1234-01-23 12:34:56.789 GMT+01:00 I abc: Log message" => LogEntry {
        timestamp: FixedOffset::east(1 * 3600).ymd(1234, 1, 23).and_hms_milli(12, 34, 56, 789).to_string(),
        level: Some(LogLevel::Info),
        meta: PlatformMetadata::AndroidLogger("1.23.4".to_owned(), "5678".to_owned(), "abc".to_owned()),
        message: "Log message".to_owned(),
    }; "basic")]
    #[test_case("[1.23.4] [5678 ] 1234-01-23 12:34:56.789 GMT+01:00 I abc: Log message\ncontinues here!" => LogEntry {
        timestamp: FixedOffset::east(1 * 3600).ymd(1234, 1, 23).and_hms_milli(12, 34, 56, 789).to_string(),
        level: Some(LogLevel::Info),
        meta: PlatformMetadata::AndroidLogger("1.23.4".to_owned(), "5678".to_owned(), "abc".to_owned()),
        message: "Log message\ncontinues here!".to_owned(),
    }; "multiline")]
    #[test_case("[1.23.4] [5678 ] 1234-01-23 12:34:56.789 ABC I abc: Log message" => LogEntry {
        timestamp: NaiveDate::from_ymd(1234, 1, 23).and_hms_milli(12, 34, 56, 789).to_string() + " ABC",
        level: Some(LogLevel::Info),
        meta: PlatformMetadata::AndroidLogger("1.23.4".to_owned(), "5678".to_owned(), "abc".to_owned()),
        message: "Log message".to_owned(),
    }; "timestamp not in GMT+hh:mm format")]
    fn logger_entry_ok(input: &str) -> LogEntry {
        parsing_test(logger_entry, input)
    }

    #[test]
    fn content_ok_logcat_empty_logger_multiple() {
        let (remainder, result) = content("========= LOGCAT ==========\n========= LOGGER ==========\n[1.23.4] [5678 ] 1234-01-23 12:34:56.789 GMT+01:00 I abc: Log message\n[1.23.4] [5678 ] 1234-01-23 12:34:56.790 GMT+01:00 W abc: Log message 2").unwrap();
        assert_eq!(remainder, "", "remainder should be empty");
        assert_eq!(
            result,
            Content {
                information: vec![],
                logs: vec![
                    Section {
                        name: LOGCAT_SECTION_NAME.to_owned(),
                        content: vec![],
                        subsections: vec![],
                    },
                    Section {
                        name: LOGGER_SECTION_NAME.to_owned(),
                        content: vec![
                            LogEntry {
                                timestamp: FixedOffset::east(1 * 3600)
                                    .ymd(1234, 1, 23)
                                    .and_hms_milli(12, 34, 56, 789)
                                    .to_string(),
                                level: Some(LogLevel::Info),
                                meta: PlatformMetadata::AndroidLogger(
                                    "1.23.4".to_owned(),
                                    "5678".to_owned(),
                                    "abc".to_owned(),
                                ),
                                message: "Log message".to_owned(),
                            },
                            LogEntry {
                                timestamp: FixedOffset::east(1 * 3600)
                                    .ymd(1234, 1, 23)
                                    .and_hms_milli(12, 34, 56, 790)
                                    .to_string(),
                                level: Some(LogLevel::Warn),
                                meta: PlatformMetadata::AndroidLogger(
                                    "1.23.4".to_owned(),
                                    "5678".to_owned(),
                                    "abc".to_owned(),
                                ),
                                message: "Log message 2".to_owned(),
                            },
                        ],
                        subsections: vec![],
                    },
                ],
            }
        );
    }
}
