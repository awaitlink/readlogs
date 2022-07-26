use std::borrow::Cow;

use crate::parsers::LogEntry;

pub fn collapse_log_entries(entries: Vec<LogEntry>) -> Vec<LogEntry> {
    let mut first_entry_of_group: Option<Cow<'_, LogEntry>> = None;

    let mut entries: Vec<_> = entries
        .iter()
        .filter_map(|entry| match &mut first_entry_of_group {
            Some(first_entry) => {
                if first_entry.timestamp == entry.timestamp
                    && first_entry.level == entry.level
                    && first_entry.meta == entry.meta
                {
                    first_entry.to_mut().message =
                        first_entry.message.clone() + "\n" + &entry.message;
                    None
                } else {
                    let result = first_entry.clone().into_owned();
                    first_entry_of_group = Some(Cow::Borrowed(entry));
                    Some(result)
                }
            }
            None => {
                first_entry_of_group = Some(Cow::Borrowed(entry));
                None
            }
        })
        .collect();

    if let Some(entry) = first_entry_of_group {
        entries.push(entry.into_owned());
    }

    entries
}

#[cfg(test)]
mod tests {
    use chrono::prelude::*;

    use super::*;
    use crate::{parsers::PlatformMetadata, LogLevel};

    #[test]
    fn collapse_log_entries_ok_android_logcat() {
        let entry1 = LogEntry {
            timestamp: Utc
                .ymd(1234, 1, 22)
                .and_hms_milli(12, 34, 56, 789)
                .to_string(),
            level: Some(LogLevel::Info),
            meta: PlatformMetadata::AndroidLogcat {
                process_id: "12345".to_owned(),
                thread_id: "12367".to_owned(),
                tag: "abc".to_owned(),
            },
            message: "Part 1".to_owned(),
        };

        let entries = vec![
            LogEntry {
                message: "Part 1".to_owned(),
                ..entry1.clone()
            },
            LogEntry {
                message: "Part 2".to_owned(),
                ..entry1.clone()
            },
            LogEntry {
                message: "Part 3".to_owned(),
                ..entry1.clone()
            },
        ];

        let result = collapse_log_entries(entries);

        assert_eq!(
            result,
            vec![LogEntry {
                message: "Part 1\nPart 2\nPart 3".to_owned(),
                ..entry1
            }]
        );
    }
}
