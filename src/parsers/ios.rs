use chrono::prelude::*;
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, take_until},
    character::complete::{multispace0, space0},
    combinator::{map, opt, verify},
    multi::many0,
    sequence::{preceded, terminated, tuple},
    IResult,
};

use crate::{parsers::*, LogLevel};

const DEFAULT_LOGS_SECTION_NAME: &str = "Logs";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntryMetadata {
    pub file: String,
    pub line: String,
    pub symbol: String,
}

#[traceable_parser]
fn level(input: Span) -> IResult<Span, LogLevel> {
    map(is_a("💙💚💛🧡❤️"), |heart: Span| {
        heart.parse().unwrap()
    })(input)
}

type Metadata = (DateTime<Utc>, Option<LogLevel>, Option<LogEntryMetadata>);

#[traceable_parser]
fn metadata(input: Span) -> IResult<Span, Metadata> {
    let verifier = |s: &Span| !s.contains('\n');

    let (remainder, (dt, _, lvl, meta)) = tuple((
        common::naive_date_time(None, "/", " ", ":", Some(":"), None),
        space0,
        opt(terminated(level, space0)),
        opt(tuple((
            tag("["),
            take_until(":"),
            tag(":"),
            is_not(" ]"),
            space0,
            alt((
                verify(take_until("]:"), verifier),
                verify(take_until("] "), verifier),
            )),
            alt((tag("]:"), tag("] "))),
        ))),
    ))(input)?;

    Ok((
        remainder,
        (
            DateTime::<Utc>::from_utc(dt, Utc),
            lvl,
            meta.map(|(_, file, _, line, _, symbol, _)| LogEntryMetadata {
                file: file.fragment().to_string(),
                line: line.fragment().to_string(),
                symbol: symbol.fragment().to_string(),
            }),
        ),
    ))
}

#[traceable_parser]
fn log_entry(input: Span) -> IResult<Span, LogEntry> {
    map(
        tuple((metadata, space0, common::message(metadata))),
        |((dt, lvl, meta), _, message)| LogEntry {
            timestamp: dt.to_string(),
            level: lvl,
            meta: PlatformMetadata::Ios(meta),
            message,
        },
    )(input)
}

#[traceable_parser]
pub fn content(input: Span) -> IResult<Span, Content> {
    preceded(
        multispace0,
        map(many0(log_entry), |logs| Content {
            information: vec![],
            logs: vec![Section {
                name: DEFAULT_LOGS_SECTION_NAME.to_owned(),
                content: logs,
                subsections: vec![],
            }],
        }),
    )(input)
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;
    use crate::test_parsing;

    fn test_timestamp(milliseconds: u32) -> DateTime<Utc> {
        Utc.ymd(1234, 1, 23).and_hms_milli(12, 34, 56, milliseconds)
    }

    fn test_metadata(line: u32) -> Option<LogEntryMetadata> {
        Some(LogEntryMetadata {
            file: "Item.abc".to_owned(),
            line: line.to_string(),
            symbol: "-[Item handleSomething]".to_owned(),
        })
    }

    fn test_log_message(
        milliseconds: u32,
        level: Option<LogLevel>,
        metadata: Option<LogEntryMetadata>,
        message: &str,
    ) -> LogEntry {
        LogEntry {
            timestamp: test_timestamp(milliseconds).to_string(),
            level,
            meta: PlatformMetadata::Ios(metadata),
            message: message.to_owned(),
        }
    }

    #[test_case(
        "1234/01/23 12:34:56:789 💚 [Item.abc:123 -[Item handleSomething]]:",
        (test_timestamp(789), Some(LogLevel::Debug), test_metadata(123));
        "basic"
    )]
    #[test_case(
        "1234/01/23 12:34:56:789      ❤️   [Item.abc:123 -[Item handleSomething]]:",
        (test_timestamp(789), Some(LogLevel::Error), test_metadata(123));
        "multiple spaces"
    )]
    #[test_case(
        "1234/01/23 12:34:56:789 💛 [Item.abc:123 -[Item handleSomething]] ",
        (test_timestamp(789), Some(LogLevel::Info), test_metadata(123));
        "meta does not have colon at the end"
    )]
    #[test_case(
        "1234/01/23 12:34:56:789 💛 [path/to/item.abc:123] ",
        (test_timestamp(789), Some(LogLevel::Info), Some(LogEntryMetadata {
            file: "path/to/item.abc".to_owned(),
            line: "123".to_owned(),
            symbol: "".to_owned(),
        }));
        "meta does not have colon at the end and does not have symbol"
    )]
    #[test_case(
        "1234/01/23 12:34:56:789",
        (test_timestamp(789), None, None);
        "no log level, no meta"
    )]
    fn metadata_ok(input: &str, output: Metadata) {
        test_parsing(metadata, input, "", output);
    }

    #[test_case(
        "Debug message\n1234/01/23 12:34:56:789 💛 Another message...",
        "1234/01/23 12:34:56:789 💛 Another message...", "Debug message".to_owned();
        "single line, more remain"
    )]
    #[test_case(
        "Debug message\n1234/01/23 12:34:56:789  Another message...",
        "1234/01/23 12:34:56:789  Another message...", "Debug message".to_owned();
        "single line, more remain, but no log level"
    )]
    #[test_case(
        "Debug message",
        "", "Debug message".to_owned();
        "single line, no more remain"
    )]
    #[test_case(
        "Debug message\n",
        "", "Debug message".to_owned();
        "single line, no more remain, trailing newline"
    )]
    #[test_case(
        "Debug message that spans\nmultiple lines {\n\ta: b,\n\tc: d,\n}\n1234/01/23 12:34:56:789 💛 Another message...",
        "1234/01/23 12:34:56:789 💛 Another message...", "Debug message that spans\nmultiple lines {\n\ta: b,\n\tc: d,\n}".to_owned();
        "multiline, more remain"
    )]
    #[test_case(
        "Debug message that spans\nmultiple lines {\n\ta: b,\n\tc: d,\n}",
        "", "Debug message that spans\nmultiple lines {\n\ta: b,\n\tc: d,\n}".to_owned();
        "multiline, no more remain"
    )]
    #[test_case(
        "Debug message that spans\nmultiple lines {\n\ta: b,\n\tc: d,\n}\n",
        "", "Debug message that spans\nmultiple lines {\n\ta: b,\n\tc: d,\n}".to_owned();
        "multiline, no more remain, trailing newline"
    )]
    fn message_ok(input: &str, remainder: &str, output: String) {
        test_parsing(common::message(metadata), input, remainder, output);
    }

    #[test_case(
        "1234/01/23 12:34:56:789 💚 [Item.abc:123 -[Item handleSomething]]: Debug message\n1234/01/23 12:34:56:789 💛 [Item.abc:123 -[Item handleSomething]]: Another message...",
        "1234/01/23 12:34:56:789 💛 [Item.abc:123 -[Item handleSomething]]: Another message...",
        test_log_message(789, Some(LogLevel::Debug), test_metadata(123), "Debug message");
        "single line, more remain"
    )]
    #[test_case(
        "1234/01/23 12:34:56:789 💚 [Item.abc:123 -[Item handleSomething]]: Debug message that spans\nmultiple lines {\n\ta: b,\n\tc: d,\n}\n1234/01/23 12:34:56:789 💛 Another message...",
        "1234/01/23 12:34:56:789 💛 Another message...",
        test_log_message(789, Some(LogLevel::Debug), test_metadata(123), "Debug message that spans\nmultiple lines {\n\ta: b,\n\tc: d,\n}");
        "multiline, more remain"
    )]
    #[test_case(
        "1234/01/23 12:34:56:123  ❤️ [Item.abc:123 -[Item handleSomething]]: Test 1\n1234/01/23 12:34:56:789  -[Abc def]:123 test",
        "1234/01/23 12:34:56:789  -[Abc def]:123 test", test_log_message(123, Some(LogLevel::Error), test_metadata(123), "Test 1");
        "next has no meta"
    )]
    #[test_case(
        "1234/01/23 12:34:56:789  Just a message.\n1234/01/23 12:34:56:987  💚 Next message",
        "1234/01/23 12:34:56:987  💚 Next message", test_log_message(789, None, None, "Just a message.");
        "no meta"
    )]
    #[test_case(
        "1234/01/23 12:34:56:789  ❤️ [Item.abc:123 -[Item handleSomething]]: \n1234/01/23 12:34:56:987  💚 Next message",
        "1234/01/23 12:34:56:987  💚 Next message", test_log_message(789, Some(LogLevel::Error), test_metadata(123), "");
        "no message"
    )]
    fn log_entry_ok(input: &str, remainder: &str, output: LogEntry) {
        test_parsing(log_entry, input, remainder, output)
    }

    #[test_case(
        "1234/01/23 12:34:56:789 💚 [Item.abc:123 -[Item handleSomething]]: Debug message that spans\nmultiple lines {\n\ta: b,\n\tc: d,\n}\n1234/01/23 12:34:56:987 💛 [Item.abc:456 -[Item handleSomething]]: Another message...",
        vec![
            test_log_message(789, Some(LogLevel::Debug), test_metadata(123), "Debug message that spans\nmultiple lines {\n\ta: b,\n\tc: d,\n}"),
            test_log_message(987, Some(LogLevel::Info), test_metadata(456), "Another message..."),
        ];
        "two log messages"
    )]
    #[test_case(
        "\n\t\n  \r\n1234/01/23 12:34:56:789 💚 [Item.abc:123 -[Item handleSomething]]: Debug message that spans\nmultiple lines {\n\ta: b,\n\tc: d,\n}\n1234/01/23 12:34:56:987 💛 [Item.abc:456 -[Item handleSomething]]: Another message...",
        vec![
            test_log_message(789, Some(LogLevel::Debug), test_metadata(123), "Debug message that spans\nmultiple lines {\n\ta: b,\n\tc: d,\n}"),
            test_log_message(987, Some(LogLevel::Info), test_metadata(456), "Another message..."),
        ];
        "starts with multispace"
    )]
    #[test_case(
        "1234/01/23 12:34:56:123  ❤️ [Item.abc:123 -[Item handleSomething]]: Test 1\n1234/01/23 12:34:56:789  -[Abc def]:123 test\n1234/01/23 12:34:56:987  💚 [Item.abc:456 -[Item handleSomething]]: Test 2",
        vec![
            test_log_message(123, Some(LogLevel::Error), test_metadata(123), "Test 1"),
            test_log_message(789, None, None, "-[Abc def]:123 test"),
            test_log_message(987, Some(LogLevel::Debug), test_metadata(456), "Test 2"),
        ];
        "no log level in the middle"
    )]
    fn content_ok(input: &str, output: Vec<LogEntry>) {
        test_parsing(
            content,
            input,
            "",
            Content {
                information: vec![],
                logs: vec![Section {
                    name: DEFAULT_LOGS_SECTION_NAME.to_owned(),
                    content: output,
                    subsections: vec![],
                }],
            },
        )
    }
}
