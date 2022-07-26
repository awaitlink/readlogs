use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, take_till, take_until, take_while},
    character::complete::{self, digit1, multispace0, newline, space0, space1},
    combinator::{map, not, opt, peek, success, value, verify},
    multi::{count, many0, many1, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, tuple},
    IResult,
};

use crate::{parsers::*, post_processing, remote_object};

const LOGCAT_SECTION_NAME: &str = "LOGCAT";
const LOGGER_SECTION_NAME: &str = "LOGGER";

#[derive(Debug, Clone, Copy, PartialEq)]
enum SectionLevel {
    Base,
    Sub,
}

#[traceable_parser]
fn subsection_header(input: Span) -> IResult<Span, &str> {
    map(
        preceded(pair(many1(tag("-")), tag(" ")), is_not("\n")),
        |span: Span| *span.fragment(),
    )(input)
}

#[traceable_parser]
fn thread(input: Span) -> IResult<Span, InfoEntry> {
    map(
        separated_pair(delimited(tag("["), digit1, tag("]")), space1, is_not("\n")),
        |(n, s): (Span, Span)| {
            InfoEntry::KeyValue(
                n.fragment().to_string(),
                Value::Generic(s.fragment().to_string()),
            )
        },
    )(input)
}

#[traceable_parser]
fn jobs_inline_section(input: Span) -> IResult<Span, Section<InfoEntry>> {
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
            name: name.fragment().to_string(),
            content: pairs,
            subsections: vec![],
        },
    )(input)
}

#[derive(Clone, Copy, Debug)]
enum IndentedSectionType {
    LocalMetrics,
    NotificationProfiles,
    OwnershipInfo,
}

use IndentedSectionType::*;

impl IndentedSectionType {
    fn supported_section_keys(&self) -> Vec<&'static str> {
        match self {
            LocalMetrics => vec!["count", "p50", "p90", "p99"],
            NotificationProfiles => {
                vec![
                    "Manually enabled profile",
                    "Manually enabled until",
                    "Manually disabled at",
                    "Now",
                ]
            }
            OwnershipInfo => vec![],
        }
    }

    fn supported_subsection_keys(&self) -> Vec<&'static str> {
        match self {
            LocalMetrics => vec!["p50", "p90", "p99"],
            NotificationProfiles => {
                vec![
                    "allowMentions",
                    "allowCalls",
                    "schedule enabled",
                    "schedule start",
                    "schedule end",
                    "schedule days",
                ]
            }
            OwnershipInfo => vec!["reserved", "unreserved"],
        }
    }

    fn supports_key_in_section(&self, key: &str) -> bool {
        self.supported_section_keys().contains(&key)
    }

    fn supports_key_in_subsection(&self, key: &str) -> bool {
        self.supported_subsection_keys().contains(&key)
    }

    fn section_keyvalues_count(&self) -> usize {
        self.supported_section_keys().len()
    }

    fn subsection_keyvalues_count(&self) -> usize {
        self.supported_subsection_keys().len()
    }
}

#[traceable_configurable_parser]
fn indented_subsection(
    ty: IndentedSectionType,
) -> impl FnMut(Span) -> IResult<Span, Section<InfoEntry>> {
    map(
        tuple((
            is_not("\n"),
            count(
                common::multispaced0(verify(common::key_maybe_enabled_value, move |entry| {
                    match entry {
                        InfoEntry::KeyValue(k, _) => ty.supports_key_in_subsection(k.as_str()),
                        _ => false,
                    }
                })),
                ty.subsection_keyvalues_count(),
            ),
        )),
        |(name, content)| Section {
            name: name.fragment().to_string(),
            content,
            subsections: vec![],
        },
    )(input)
}

#[traceable_configurable_parser]
fn subsection_with_indented_subsections<'a>(
    raw_name: &'a str,
    name: &'a str,
    explicit_none: &'a str,
    ty: IndentedSectionType,
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, Vec<Section<InfoEntry>>> {
    preceded(
        common::multispaced0(tag(raw_name)),
        map(
            pair(
                opt(common::multispaced0(tag(explicit_none))),
                many0(common::multispaced0(indented_subsection(ty))),
            ),
            |(explicit_none, subsections)| {
                vec![Section {
                    name: name.to_owned(),
                    content: match explicit_none {
                        Some(_) => vec![InfoEntry::ExplicitNone],
                        None => vec![],
                    },
                    subsections,
                }]
            },
        ),
    )(input)
}

#[traceable_configurable_parser]
fn section_with_indented_subsections(
    ty: IndentedSectionType,
) -> impl FnMut(Span) -> IResult<Span, Section<InfoEntry>> {
    map(
        tuple((
            is_not("\n"),
            count(
                common::multispaced0(verify(common::key_maybe_enabled_value, move |entry| {
                    match entry {
                        InfoEntry::KeyValue(k, _) => ty.supports_key_in_section(k.as_str()),
                        _ => false,
                    }
                })),
                ty.section_keyvalues_count(),
            ),
            many0(common::multispaced0(indented_subsection(ty))),
        )),
        |(name, content, subsections)| Section {
            name: name.fragment().to_string(),
            content,
            subsections,
        },
    )(input)
}

#[traceable_parser]
fn generic_table(input: Span) -> IResult<Span, GenericTable> {
    let row = |input| {
        delimited(
            tag("|"),
            map(
                separated_list1(tag("|"), is_not("|\n")),
                |items: Vec<Span>| {
                    items
                        .iter()
                        .map(|s| s.trim().to_owned())
                        .collect::<Vec<_>>()
                },
            ),
            tag("|"),
        )(input)
    };

    map(
        separated_pair(
            terminated(row, newline),
            tuple((many1(pair(tag("|"), many1(tag("-")))), tag("|"), newline)),
            many0(terminated(row, opt(newline))),
        ),
        |(header, rows)| GenericTable { header, rows },
    )(input)
}

#[traceable_configurable_parser]
fn info_section<'a>(depth: SectionLevel) -> impl FnMut(Span) -> IResult<Span, Section<InfoEntry>> {
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
                map(generic_table, |table| vec![InfoEntry::GenericTable(table)]),
                many1(common::multispaced0(common::key_maybe_enabled_value)),
                many1(common::multispaced0(thread)),
                map(remote_object, |ro| vec![InfoEntry::RemoteObject(ro)]),
                value(vec![InfoEntry::ExplicitNone], tag("None")),
                many1(common::multispaced0(map(
                    preceded(
                        peek(not(alt((
                            section_with_indented_subsections(LocalMetrics),
                            section_with_indented_subsections(NotificationProfiles),
                            // section_with_indented_subsections(OwnershipInfo), // TODO: Investigate, it causes parsers::android::tests::info_section_ok::blocked_threads failure
                        )))),
                        is_not("\n-="),
                    ),
                    |s: Span| InfoEntry::Generic(s.fragment().to_string()),
                ))),
            ))),
        ),
        success(vec![]),
    ))(remainder)?;

    let (remainder, subsections) = match depth {
        SectionLevel::Base => alt((
            many1(info_section(SectionLevel::Sub)),
            many1(common::multispaced0(section_with_indented_subsections(
                LocalMetrics,
            ))),
            subsection_with_indented_subsections(
                "Profiles:",
                "Profiles",
                "No notification profiles",
                NotificationProfiles,
            ),
            subsection_with_indented_subsections(
                "Ownership Info:",
                "Ownership Info",
                "No ownership info to display.",
                OwnershipInfo,
            ),
            success(vec![]),
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

#[traceable_configurable_parser]
fn logcat_entry<'a>(year: i32) -> impl FnMut(Span) -> IResult<Span, LogEntry> {
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
            verify(
                terminated(
                    take_till(char::is_whitespace),
                    pair(take_while(char::is_whitespace), opt(tag(": "))),
                ),
                |span: &Span| !span.contains('\n'),
            ),
            space0,
            alt((is_not("\n"), success(span("")))),
        )),
        |(dt, _, process_id, _, thread_id, _, level, _, tag, _, message)| LogEntry {
            timestamp: dt.to_string(),
            level: Some(level.parse().unwrap()),
            meta: PlatformMetadata::AndroidLogcat {
                process_id: process_id.fragment().to_string(),
                thread_id: thread_id.fragment().to_string(),
                tag: tag.trim_end_matches(':').trim().to_owned(),
            },
            message: message.fragment().to_string(),
        },
    )(input)
}

#[traceable_configurable_parser]
fn logcat_section<'a>(year: i32) -> impl FnMut(Span) -> IResult<Span, Section<LogEntry>> {
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

#[traceable_parser]
fn logger_metadata(input: Span) -> IResult<Span, (PlatformMetadata, String, LogLevel)> {
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
        map(take_until(" "), |span: Span| {
            LoggerTimezone::Unparsed(span.fragment())
        }),
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
                PlatformMetadata::AndroidLogger {
                    version: version.fragment().to_string(),
                    thread_id: thread_id.trim().to_owned(),
                    tag: tag.trim().to_owned(),
                },
                match tz {
                    LoggerTimezone::Parsed(tz) => tz.from_local_datetime(&dt).unwrap().to_string(),
                    LoggerTimezone::Unparsed(s) => dt.to_string() + " " + s,
                },
                level.parse().unwrap(),
            )
        },
    )(input)
}

#[traceable_parser]
fn logger_entry(input: Span) -> IResult<Span, LogEntry> {
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

#[traceable_parser]
pub fn content(input: Span) -> IResult<Span, Content> {
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
    use crate::{test_parsing, test_parsing_err_or_remainder};

    #[test_case("-- Abc", "Abc"; "basic")]
    #[test_case("--------- Long line", "Long line"; "long")]
    fn subsection_header_ok(input: &str, output: &str) {
        test_parsing(subsection_header, input, "", output);
    }

    #[test_case(
        "id: JOB::abcd1234-efgh-5678-ijkl-9012mnop1234 | a: TestJob | b: _test_value_ | number: 123 | negative: -1", Section {
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
        "jobSpecId: JOB::abcd1234-efgh-5678-ijkl-9012mnop1234 | a: TestConstraint | anotherValue: false", Section {
            name: "JOB::abcd1234-efgh-5678-ijkl-9012mnop1234".to_owned(),
            content: vec![
                InfoEntry::KeyValue("a".to_owned(), Value::Generic("TestConstraint".to_owned())),
                InfoEntry::KeyValue("anotherValue".to_owned(), Value::Generic("false".to_owned())),
            ],
            subsections: vec![],
        };
        "constraint"
    )]
    fn jobs_inline_section_ok(input: &str, output: Section<InfoEntry>) {
        test_parsing(jobs_inline_section, input, "", output);
    }

    #[test_case(
        "========= HEADER =========\nTime          : 1234567890123\nDays Installed: 123\nself.isRegistered()  : true",
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
        "========== JOBS ===========\n-- Jobs\nid: JOB::abcd1234-efgh-5678-ijkl-9012mnop1234 | a: TestJob | b: _test_value_ | number: 123 | negative: -1\n\n-- Constraints\njobSpecId: JOB::abcd1234-efgh-5678-ijkl-9012mnop1234 | a: TestConstraint | anotherValue: false\n\n-- Dependencies\nNone",
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
                    content: vec![InfoEntry::ExplicitNone],
                    subsections: vec![],
                },
            ],
        }; "jobs"
    )]
    #[test_case(
        "====== HEADER =======\n-- Abc\nABC123          : true\nCapability Name : false\n\n-- Def\nABC123          : SUPPORTED\nCapability Name : NOT_SUPPORTED\nexample.testFlag: 1:2,3:4,*:5",
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
        "====== LOCAL METRICS ======\ncold-start-conversation-list\n  count: 5\n  p50: 3456\n  p90: 4567\n  p99: 4567\n    application-create\n      p50: 123\n      p90: 456\n      p99: 456\n    data-loaded\n      p50: 456\n      p90: 789\n      p99: 789\n\n\nconversation-open\n  count: 123\n  p50: 1234\n  p90: 5678\n  p99: 12345",
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
        "======== PIN STATE ========\nKey: abc_def_ghi\nTest Value: 1234567890\nAbcDef: true",
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
        "========== POWER ==========\nCurrent bucket: Frequent\nHighest bucket: Active\nLowest bucket : Rare\n\nMon Jan 23 12:34:56 GMT+01:00 1234: Bucket Change: Active\nMon Jan 23 12:34:57 GMT+01:00 1234: Bucket Change: Rare\nMon Jan 23 12:34:58 GMT+01:00 1234: Bucket Change: Frequent",
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
        "====== NOTIFICATIONS ======\nTest key        : true\nAnother test key: false\n\n-- abc_def_v2\ntest       : LOW (2)\nanotherTest: N/A (Requires API 30)\n\n-- abc_def : 123\ntest       : value",
        Section {
            name: "NOTIFICATIONS".to_owned(),
            content: vec![
                InfoEntry::KeyValue("Test key".to_owned(), Value::Generic("true".to_owned())),
                InfoEntry::KeyValue("Another test key".to_owned(), Value::Generic("false".to_owned())),
            ],
            subsections: vec![
                Section {
                    name: "abc_def_v2".to_owned(),
                    content: vec![
                        InfoEntry::KeyValue("test".to_owned(), Value::Generic("LOW (2)".to_owned())),
                        InfoEntry::KeyValue("anotherTest".to_owned(), Value::Generic("N/A (Requires API 30)".to_owned())),
                    ],
                    subsections: vec![],
                },
                Section {
                    name: "abc_def : 123".to_owned(),
                    content: vec![
                        InfoEntry::KeyValue("test".to_owned(), Value::Generic("value".to_owned())),
                    ],
                    subsections: vec![],
                },
            ],
        }; "notifications"
    )]
    #[test_case(
        "===== NOTIFICATION PROFILES =====\nManually enabled profile: 0\nManually enabled until  : 0\nManually disabled at    : 1234567890123\nNow                     : 1234567890321\n\nProfiles:\n    Profile 1\n    allowMentions   : false\n    allowCalls      : false\n    schedule enabled: false\n    schedule start  : 900\n    schedule end    : 2100\n    schedule days   : [MONDAY, TUESDAY, WEDNESDAY, THURSDAY, FRIDAY]",
        Section {
            name: "NOTIFICATION PROFILES".to_owned(),
            content: vec![
                InfoEntry::KeyValue("Manually enabled profile".to_owned(), Value::Generic("0".to_owned())),
                InfoEntry::KeyValue("Manually enabled until".to_owned(), Value::Generic("0".to_owned())),
                InfoEntry::KeyValue("Manually disabled at".to_owned(), Value::Generic("1234567890123".to_owned())),
                InfoEntry::KeyValue("Now".to_owned(), Value::Generic("1234567890321".to_owned())),
            ],
            subsections: vec![
                Section {
                    name: "Profiles".to_owned(),
                    content: vec![],
                    subsections: vec![
                        Section {
                            name: "Profile 1".to_owned(),
                            content: vec![
                                InfoEntry::KeyValue("allowMentions".to_owned(), Value::Generic("false".to_owned())),
                                InfoEntry::KeyValue("allowCalls".to_owned(), Value::Generic("false".to_owned())),
                                InfoEntry::KeyValue("schedule enabled".to_owned(), Value::Generic("false".to_owned())),
                                InfoEntry::KeyValue("schedule start".to_owned(), Value::Generic("900".to_owned())),
                                InfoEntry::KeyValue("schedule end".to_owned(), Value::Generic("2100".to_owned())),
                                InfoEntry::KeyValue("schedule days".to_owned(), Value::Generic("[MONDAY, TUESDAY, WEDNESDAY, THURSDAY, FRIDAY]".to_owned())),
                            ],
                            subsections: vec![],
                        }
                    ],
                }
            ],
        }; "notification profiles"
    )]
    #[test_case(
        "===== NOTIFICATION PROFILES =====\nManually enabled profile: 0\nManually enabled until  : 0\nManually disabled at    : 1234567890123\nNow                     : 1234567890321\n\nProfiles:\n    No notification profiles",
        Section {
            name: "NOTIFICATION PROFILES".to_owned(),
            content: vec![
                InfoEntry::KeyValue("Manually enabled profile".to_owned(), Value::Generic("0".to_owned())),
                InfoEntry::KeyValue("Manually enabled until".to_owned(), Value::Generic("0".to_owned())),
                InfoEntry::KeyValue("Manually disabled at".to_owned(), Value::Generic("1234567890123".to_owned())),
                InfoEntry::KeyValue("Now".to_owned(), Value::Generic("1234567890321".to_owned())),
            ],
            subsections: vec![
                Section {
                    name: "Profiles".to_owned(),
                    content: vec![InfoEntry::ExplicitNone],
                    subsections: vec![],
                }
            ],
        }; "notification profiles empty"
    )]
    #[test_case(
        "======== EXOPLAYER POOL =========\nTotal players created: 0\nMax allowed unreserved instances: 12\nMax allowed reserved instances: 1\nAvailable created unreserved instances: 0\nAvailable created reserved instances: 0\nTotal unreserved created: 0\nTotal reserved created: 0\n\nOwnership Info:\n  No ownership info to display.",
        Section {
            name: "EXOPLAYER POOL".to_owned(),
            content: vec![
                InfoEntry::KeyValue("Total players created".to_owned(), Value::Generic("0".to_owned())),
                InfoEntry::KeyValue("Max allowed unreserved instances".to_owned(), Value::Generic("12".to_owned())),
                InfoEntry::KeyValue("Max allowed reserved instances".to_owned(), Value::Generic("1".to_owned())),
                InfoEntry::KeyValue("Available created unreserved instances".to_owned(), Value::Generic("0".to_owned())),
                InfoEntry::KeyValue("Available created reserved instances".to_owned(), Value::Generic("0".to_owned())),
                InfoEntry::KeyValue("Total unreserved created".to_owned(), Value::Generic("0".to_owned())),
                InfoEntry::KeyValue("Total reserved created".to_owned(), Value::Generic("0".to_owned())),
            ],
            subsections: vec![
                Section {
                    name: "Ownership Info".to_owned(),
                    content: vec![InfoEntry::ExplicitNone],
                    subsections: vec![],
                }
            ],
        }; "exoplayer pool (empty)"
    )]
    #[test_case(
        "======== EXOPLAYER POOL =========\nTotal players created: 0\nMax allowed unreserved instances: 12\nMax allowed reserved instances: 1\nAvailable created unreserved instances: 0\nAvailable created reserved instances: 0\nTotal unreserved created: 0\nTotal reserved created: 0\n\nOwnership Info:\n  Owner abc def\n    reserved: 12\n    unreserved: 1\n  Owner abc def ghi\n    reserved: 5\n    unreserved: 4\n",
        Section {
            name: "EXOPLAYER POOL".to_owned(),
            content: vec![
                InfoEntry::KeyValue("Total players created".to_owned(), Value::Generic("0".to_owned())),
                InfoEntry::KeyValue("Max allowed unreserved instances".to_owned(), Value::Generic("12".to_owned())),
                InfoEntry::KeyValue("Max allowed reserved instances".to_owned(), Value::Generic("1".to_owned())),
                InfoEntry::KeyValue("Available created unreserved instances".to_owned(), Value::Generic("0".to_owned())),
                InfoEntry::KeyValue("Available created reserved instances".to_owned(), Value::Generic("0".to_owned())),
                InfoEntry::KeyValue("Total unreserved created".to_owned(), Value::Generic("0".to_owned())),
                InfoEntry::KeyValue("Total reserved created".to_owned(), Value::Generic("0".to_owned())),
            ],
            subsections: vec![
                Section {
                    name: "Ownership Info".to_owned(),
                    content: vec![],
                    subsections: vec![
                        Section {
                            name: "Owner abc def".to_owned(),
                            content: vec![
                                InfoEntry::KeyValue("reserved".to_owned(), Value::Generic("12".to_owned())),
                                InfoEntry::KeyValue("unreserved".to_owned(), Value::Generic("1".to_owned())),
                            ],
                            subsections: vec![],
                        },
                        Section {
                            name: "Owner abc def ghi".to_owned(),
                            content: vec![
                                InfoEntry::KeyValue("reserved".to_owned(), Value::Generic("5".to_owned())),
                                InfoEntry::KeyValue("unreserved".to_owned(), Value::Generic("4".to_owned())),
                            ],
                            subsections: vec![],
                        },
                    ],
                },
            ],
        }; "exoplayer pool (with ownership info)"
    )]
    #[test_case(
        "========== TRACE ==========\nhttps://debuglogs.org/0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123",
        Section {
            name: "TRACE".to_owned(),
            content: vec![InfoEntry::RemoteObject(
                RemoteObject::new_unchecked(Platform::Android, None, "0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123")
            )],
            subsections: vec![],
        }; "trace"
    )]
    #[test_case(
        "========= THREADS =========\n[1] main\n[1234] Signal Catcher\n[1235] AbcDefGhi\n[6789] OkHttp https://abc-def.example.org/...",
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
        "===== BLOCKED THREADS =====\n-- [9876] AbcDefGhi (BLOCKED)\nghi.jkl.ABCdef.abcDefGhi(Native Method)\nabc.def.Abc$Cba.run(DEF.java:456)\nabc.def.Def.run(ABC.java:123)",
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
        "===== REMAPPED RECORDS =====\n--- Recipients\n\n| _id | old_id | new_id |\n|-----|--------|--------|\n| 1   | 23     | 456    |\n| 2   | 345    | 678    |\n\n--- Threads\n\n| _id | old_id | new_id |\n|-----|--------|--------|\n| 3   | 45     | 678    |\n| 4   | 567    | 890    |",
        Section {
            name: "REMAPPED RECORDS".to_owned(),
            content: vec![],
            subsections: vec![
                Section {
                    name: "Recipients".to_owned(),
                    content: vec![
                        InfoEntry::GenericTable(GenericTable {
                            header: vec!["_id".to_owned(), "old_id".to_owned(), "new_id".to_owned()],
                            rows: vec![
                                vec!["1".to_owned(), "23".to_owned(), "456".to_owned()],
                                vec!["2".to_owned(), "345".to_owned(), "678".to_owned()],
                            ]
                        }),
                    ],
                    subsections: vec![],
                },
                Section {
                    name: "Threads".to_owned(),
                    content: vec![
                        InfoEntry::GenericTable(GenericTable {
                            header: vec!["_id".to_owned(), "old_id".to_owned(), "new_id".to_owned()],
                            rows: vec![
                                vec!["3".to_owned(), "45".to_owned(), "678".to_owned()],
                                vec!["4".to_owned(), "567".to_owned(), "890".to_owned()],
                            ]
                        }),
                    ],
                    subsections: vec![],
                }
            ],
        }; "remapped records"
    )]
    #[test_case(
        "\n\n \n====== EMPTY SECTION ======\n",
        Section {
            name: "EMPTY SECTION".to_owned(),
            content: vec![],
            subsections: vec![],
        }; "empty section"
    )]
    fn info_section_ok(input: &str, output: Section<InfoEntry>) {
        test_parsing(info_section(SectionLevel::Base), input, "", output);
    }

    #[test_case("01-23 12:34:56.789 12345 12367 I abc: Log message", LogEntry {
        timestamp: NaiveDate::from_ymd(1234, 1, 23).and_hms_milli(12, 34, 56, 789).to_string(),
        level: Some(LogLevel::Info),
        meta: PlatformMetadata::AndroidLogcat { process_id: "12345".to_owned(), thread_id: "12367".to_owned(), tag: "abc".to_owned() },
        message: "Log message".to_owned(),
    }; "basic")]
    #[test_case("01-23 12:34:56.789 12345 12367 I V...@... MSG_WINDOW_FOCUS_CHANGED 1 1", LogEntry {
        timestamp: NaiveDate::from_ymd(1234, 1, 23).and_hms_milli(12, 34, 56, 789).to_string(),
        level: Some(LogLevel::Info),
        meta: PlatformMetadata::AndroidLogcat { process_id: "12345".to_owned(), thread_id: "12367".to_owned(), tag: "V...@...".to_owned() },
        message: "MSG_WINDOW_FOCUS_CHANGED 1 1".to_owned(),
    }; "no colon separator for tag")]
    #[test_case("01-23 12:34:56.789 12345 12367 I V...@... Log message: test", LogEntry {
        timestamp: NaiveDate::from_ymd(1234, 1, 23).and_hms_milli(12, 34, 56, 789).to_string(),
        level: Some(LogLevel::Info),
        meta: PlatformMetadata::AndroidLogcat { process_id: "12345".to_owned(), thread_id: "12367".to_owned(), tag: "V...@...".to_owned() },
        message: "Log message: test".to_owned(),
    }; "no colon separator for tag but has colon later")]
    #[test_case("01-23 12:34:56.789 12345 12367 I abc: ", LogEntry {
        timestamp: NaiveDate::from_ymd(1234, 1, 23).and_hms_milli(12, 34, 56, 789).to_string(),
        level: Some(LogLevel::Info),
        meta: PlatformMetadata::AndroidLogcat { process_id: "12345".to_owned(), thread_id: "12367".to_owned(), tag: "abc".to_owned() },
        message: "".to_owned(),
    }; "no message")]
    fn logcat_entry_ok(input: &str, output: LogEntry) {
        test_parsing(logcat_entry(1234), input, "", output);
    }

    #[test]
    fn logcat_entry_err() {
        let input =
        "01-24 12:34:56.789 12345 12367 I V...@... MSG_WINDOW_FOCUS_CHANGED 1 1\n============ LOGGER =============\n[1.23.4] [5678 ] 1234-01-23 12:34:56.789 GMT+01:00 I abc: Log message";

        test_parsing_err_or_remainder(logcat_entry(1234), input);
    }

    #[test_case("========= LOGCAT ==========\n--------- beginning of crash\n01-21 12:34:56.789  1234  5678 F libc    : Fatal signal 11 (SIGSEGV), code 2, fault addr 0x12345678 in tid 9876 (Abc)\n--------- beginning of main\n01-22 12:34:56.789 12345 12367 I chatty  : uid=10001(org.thoughtcrime.securesms) expire 1 line\n01-23 12:34:56.789 12345 12367 I chatty  : uid=10001(org.thoughtcrime.securesms) expire 5 lines",
    Section {
        name: LOGCAT_SECTION_NAME.to_owned(),
        content: vec![],
        subsections: vec![
            Section {
                name: "beginning of crash".to_owned(),
                content: vec![
                    LogEntry {
                        timestamp: NaiveDate::from_ymd(1234, 1, 21).and_hms_milli(12, 34, 56, 789).to_string(),
                        level: Some(LogLevel::Fatal),
                        meta: PlatformMetadata::AndroidLogcat { process_id: "1234".to_owned(), thread_id: "5678".to_owned(), tag: "libc".to_owned() },
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
                        meta: PlatformMetadata::AndroidLogcat { process_id: "12345".to_owned(), thread_id: "12367".to_owned(), tag: "chatty".to_owned() },
                        message: "uid=10001(org.thoughtcrime.securesms) expire 1 line".to_owned(),
                    },
                    LogEntry {
                        timestamp: NaiveDate::from_ymd(1234, 1, 23).and_hms_milli(12, 34, 56, 789).to_string(),
                        level: Some(LogLevel::Info),
                        meta: PlatformMetadata::AndroidLogcat { process_id: "12345".to_owned(), thread_id: "12367".to_owned(), tag: "chatty".to_owned() },
                        message: "uid=10001(org.thoughtcrime.securesms) expire 5 lines".to_owned(),
                    },
                ],
                subsections: vec![],
            },
        ],
    }; "basic")]
    fn logcat_section_ok(input: &str, output: Section<LogEntry>) {
        test_parsing(logcat_section(1234), input, "", output);
    }

    #[test_case("[1.23.4] [5678 ] 1234-01-23 12:34:56.789 GMT+01:00 I abc: Log message", LogEntry {
        timestamp: FixedOffset::east(3600).ymd(1234, 1, 23).and_hms_milli(12, 34, 56, 789).to_string(),
        level: Some(LogLevel::Info),
        meta: PlatformMetadata::AndroidLogger { version: "1.23.4".to_owned(), thread_id: "5678".to_owned(), tag: "abc".to_owned() },
        message: "Log message".to_owned(),
    }; "basic")]
    #[test_case("[1.23.4] [main ] 1234-01-23 12:34:56.789 GMT+01:00 I abc: Log message", LogEntry {
        timestamp: FixedOffset::east(3600).ymd(1234, 1, 23).and_hms_milli(12, 34, 56, 789).to_string(),
        level: Some(LogLevel::Info),
        meta: PlatformMetadata::AndroidLogger { version: "1.23.4".to_owned(), thread_id: "main".to_owned(), tag: "abc".to_owned() },
        message: "Log message".to_owned(),
    }; "main thread id")]
    #[test_case("[1.23.4] [5678 ] 1234-01-23 12:34:56.789 GMT+01:00 I abc: Log message\ncontinues here!", LogEntry {
        timestamp: FixedOffset::east(3600).ymd(1234, 1, 23).and_hms_milli(12, 34, 56, 789).to_string(),
        level: Some(LogLevel::Info),
        meta: PlatformMetadata::AndroidLogger { version: "1.23.4".to_owned(), thread_id: "5678".to_owned(), tag: "abc".to_owned() },
        message: "Log message\ncontinues here!".to_owned(),
    }; "multiline")]
    #[test_case("[1.23.4] [5678 ] 1234-01-23 12:34:56.789 ABC I abc: Log message", LogEntry {
        timestamp: NaiveDate::from_ymd(1234, 1, 23).and_hms_milli(12, 34, 56, 789).to_string() + " ABC",
        level: Some(LogLevel::Info),
        meta: PlatformMetadata::AndroidLogger { version: "1.23.4".to_owned(), thread_id: "5678".to_owned(), tag: "abc".to_owned() },
        message: "Log message".to_owned(),
    }; "timestamp not in GMT+hh:mm format")]
    fn logger_entry_ok(input: &str, output: LogEntry) {
        test_parsing(logger_entry, input, "", output);
    }

    #[test]
    fn content_ok_logcat_empty_logger_multiple() {
        test_parsing(
            content,
            "========= LOGCAT ==========\n========= LOGGER ==========\n[1.23.4] [5678 ] 1234-01-23 12:34:56.789 GMT+01:00 I abc: Log message\n[1.23.4] [5678 ] 1234-01-23 12:34:56.790 GMT+01:00 W abc: Log message 2",
            "",
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
                                timestamp: FixedOffset::east(3600)
                                    .ymd(1234, 1, 23)
                                    .and_hms_milli(12, 34, 56, 789)
                                    .to_string(),
                                level: Some(LogLevel::Info),
                                meta: PlatformMetadata::AndroidLogger {
                                    version: "1.23.4".to_owned(),
                                    thread_id: "5678".to_owned(),
                                    tag: "abc".to_owned(),
                                },
                                message: "Log message".to_owned(),
                            },
                            LogEntry {
                                timestamp: FixedOffset::east(3600)
                                    .ymd(1234, 1, 23)
                                    .and_hms_milli(12, 34, 56, 790)
                                    .to_string(),
                                level: Some(LogLevel::Warn),
                                meta: PlatformMetadata::AndroidLogger {
                                    version: "1.23.4".to_owned(),
                                    thread_id: "5678".to_owned(),
                                    tag: "abc".to_owned(),
                                },
                                message: "Log message 2".to_owned(),
                            },
                        ],
                        subsections: vec![],
                    },
                ],
            }
        );
    }

    #[test]
    fn content_ok_logcat_multiple_logger_multiple() {
        test_parsing(
            content,
            "========= LOGCAT ==========\n--------- beginning of main\n01-24 12:34:56.789 12345 12367 I V...@... MSG_WINDOW_FOCUS_CHANGED 1 1\n01-24 12:34:56.789 12345 12367 I V...@... MSG_WINDOW_FOCUS_CHANGED 1 1\n============ LOGGER =============\n[1.23.4] [5678 ] 1234-01-23 12:34:56.789 GMT+01:00 I abc: Log message\n[1.23.4] [5678 ] 1234-01-23 12:34:56.790 GMT+01:00 W abc: Log message 2",
            "",
            Content {
                information: vec![],
                logs: vec![
                    Section {
                        name: LOGCAT_SECTION_NAME.to_owned(),
                        content: vec![],
                        subsections: vec![
                            Section {
                                name: "beginning of main".to_owned(),
                                content: vec![LogEntry {
                                    timestamp: NaiveDate::from_ymd(Utc::today().year(), 1, 24)
                                        .and_hms_milli(12, 34, 56, 789)
                                        .to_string(),
                                    level: Some(LogLevel::Info),
                                    meta: PlatformMetadata::AndroidLogcat {
                                        process_id: "12345".to_owned(),
                                        thread_id: "12367".to_owned(),
                                        tag: "V...@...".to_owned(),
                                    },
                                    message: "MSG_WINDOW_FOCUS_CHANGED 1 1\nMSG_WINDOW_FOCUS_CHANGED 1 1"
                                        .to_owned(),
                                }],
                                subsections: vec![]
                            }
                        ],
                    },
                    Section {
                        name: LOGGER_SECTION_NAME.to_owned(),
                        content: vec![
                            LogEntry {
                                timestamp: FixedOffset::east(3600)
                                    .ymd(1234, 1, 23)
                                    .and_hms_milli(12, 34, 56, 789)
                                    .to_string(),
                                level: Some(LogLevel::Info),
                                meta: PlatformMetadata::AndroidLogger {
                                    version: "1.23.4".to_owned(),
                                    thread_id: "5678".to_owned(),
                                    tag: "abc".to_owned(),
                                },
                                message: "Log message".to_owned(),
                            },
                            LogEntry {
                                timestamp: FixedOffset::east(3600)
                                    .ymd(1234, 1, 23)
                                    .and_hms_milli(12, 34, 56, 790)
                                    .to_string(),
                                level: Some(LogLevel::Warn),
                                meta: PlatformMetadata::AndroidLogger {
                                    version: "1.23.4".to_owned(),
                                    thread_id: "5678".to_owned(),
                                    tag: "abc".to_owned(),
                                },
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
