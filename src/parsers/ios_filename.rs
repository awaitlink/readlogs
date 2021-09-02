use chrono::prelude::*;
use derive_more::Display;
use nom::{
    bytes::complete::{tag, take_until},
    character::complete::space1,
    combinator::rest,
    sequence::{delimited, preceded, terminated},
    IResult,
};
use strum_macros::EnumString;

use crate::{impl_from_str, parsers::common};

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString)]
pub enum AppId {
    #[strum(serialize = "org.whispersystems.signal")]
    Signal,
    #[display(fmt = "NSE")]
    #[strum(serialize = "org.whispersystems.signal.NotificationServiceExtension")]
    NotificationServiceExtension,
    #[display(fmt = "SAE")]
    #[strum(serialize = "org.whispersystems.signal.shareextension")]
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

fn log_filename(input: &str) -> IResult<&str, LogFilename> {
    let (remainder, submission_time) =
        common::naive_date_time(None, ".", " ", ".", None, None)(input)?;
    let (remainder, folder_id) = delimited(tag(" "), take_until("/"), tag("/"))(remainder)?;
    let (remainder, app_id) = terminated(take_until(" "), space1)(remainder)?;
    let (remainder, file_time) =
        common::naive_date_time(None, "-", "--", "-", Some("-"), None)(remainder)?;
    let (remainder, extension) = preceded(tag("."), rest)(remainder)?;

    Ok((
        remainder,
        LogFilename {
            submission_time,
            folder_id: folder_id.to_owned(),
            app_id: app_id.parse().unwrap(),
            file_time: DateTime::<Utc>::from_utc(file_time, Utc),
            extension: extension.to_owned(),
        },
    ))
}

impl_from_str!(log_filename => LogFilename);

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn ok() {
        let (remainder, result) = log_filename("1234.01.23 12.34.56 ABCD1234-1AB2-3CDE-456F-789AB0CD1E2F/org.whispersystems.signal 1234-01-22--06-54-32-109.log").unwrap();

        assert_eq!(remainder, "", "remainder should be empty");

        assert_eq!(
            result,
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
