use chrono::prelude::*;
use derive_more::Display;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::space1,
    combinator::{rest, value},
    sequence::{delimited, preceded, terminated},
    IResult,
};

use crate::{impl_from_str, parsers::*};

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AppId {
    Signal,
    #[display(fmt = "NSE")]
    NotificationServiceExtension,
    #[display(fmt = "SAE")]
    ShareAppExtension,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LogFilename {
    /// Seems to be 12-hour time in the timezone of the submitter,
    /// but without indication of whether it's AM or PM.
    pub submission_time: NaiveDateTime,
    pub folder_id: String,
    pub app_id: AppId,
    pub file_time: DateTime<Utc>,
    pub extension: String,
}

#[traceable_parser]
fn app_id_with_space(input: Span) -> IResult<Span, AppId> {
    preceded(
        tag("org.whispersystems.signal"),
        alt((
            value(AppId::Signal, space1),
            terminated(
                alt((
                    value(AppId::NotificationServiceExtension, tag(".SignalNSE")),
                    value(
                        AppId::NotificationServiceExtension,
                        tag(".NotificationServiceExtension"),
                    ),
                    value(AppId::ShareAppExtension, tag(".shareextension")),
                )),
                space1,
            ),
        )),
    )(input)
}

#[traceable_parser]
fn log_filename(input: Span) -> IResult<Span, LogFilename> {
    let (remainder, submission_time) =
        common::naive_date_time(None, ".", " ", ".", None, None)(input)?;
    let (remainder, folder_id) = delimited(tag(" "), take_until("/"), tag("/"))(remainder)?;
    let (remainder, app_id) = app_id_with_space(remainder)?;
    let (remainder, file_time) =
        common::naive_date_time(None, "-", "--", "-", Some("-"), None)(remainder)?;
    let (remainder, extension) = preceded(tag("."), rest)(remainder)?;

    Ok((
        remainder,
        LogFilename {
            submission_time,
            folder_id: folder_id.fragment().to_string(),
            app_id,
            file_time: DateTime::<Utc>::from_utc(file_time, Utc),
            extension: extension.fragment().to_string(),
        },
    ))
}

impl_from_str!(log_filename => LogFilename);

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;
    use crate::utils::test_parsing;

    #[test]
    fn partial_ord() {
        let a = LogFilename {
            submission_time: NaiveDate::from_ymd(1234, 1, 23).and_hms(12, 34, 56),
            folder_id: "ABCD1234-1AB2-3CDE-456F-789AB0CD1E2F".to_owned(),
            app_id: AppId::Signal,
            file_time: Utc.ymd(1234, 1, 22).and_hms_milli(6, 54, 32, 109),
            extension: "log".to_owned(),
        };

        let b = LogFilename {
            app_id: AppId::NotificationServiceExtension,
            file_time: Utc.ymd(1234, 1, 22).and_hms_milli(6, 54, 32, 111),
            ..a.clone()
        };

        let c = LogFilename {
            file_time: Utc.ymd(1234, 1, 22).and_hms_milli(6, 54, 32, 123),
            ..a.clone()
        };

        // a: Signal ...109
        // c: Signal ...123
        // b: NSE    ...111

        assert!(a < c);
        assert!(c < b);
    }

    #[test_case("org.whispersystems.signal ", AppId::Signal)]
    #[test_case(
        "org.whispersystems.signal.SignalNSE ",
        AppId::NotificationServiceExtension
    )]
    #[test_case(
        "org.whispersystems.signal.NotificationServiceExtension ",
        AppId::NotificationServiceExtension
    )]
    #[test_case("org.whispersystems.signal.shareextension ", AppId::ShareAppExtension)]
    fn app_id_with_space_ok(input: &str, output: AppId) {
        test_parsing(app_id_with_space, input, "", output)
    }

    #[test]
    fn log_filename_ok() {
        test_parsing(
            log_filename,
            "1234.01.23 12.34.56 ABCD1234-1AB2-3CDE-456F-789AB0CD1E2F/org.whispersystems.signal 1234-01-22--06-54-32-109.log",
            "",
            LogFilename {
                submission_time: NaiveDate::from_ymd(1234, 1, 23).and_hms(12, 34, 56),
                folder_id: "ABCD1234-1AB2-3CDE-456F-789AB0CD1E2F".to_owned(),
                app_id: AppId::Signal,
                file_time: Utc.ymd(1234, 1, 22).and_hms_milli(6, 54, 32, 109),
                extension: "log".to_owned(),
            }
        );
    }
}
