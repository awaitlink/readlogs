use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag},
    combinator::{map, not, opt, value, verify},
    sequence::{preceded, separated_pair, terminated, tuple},
    IResult,
};

use crate::{
    impl_from_str,
    parsers::{traceable_parser, Span},
    Platform,
};

pub const KEY_LENGTH: usize = 64;
pub const BASE_DEBUGLOGS_URL: &str = "https://debuglogs.org/";
pub const BASE_WORKER_URL: &str = "https://getlogs.warp.workers.dev/";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteObject {
    platform: Platform,
    version: Option<String>,
    key: String,
}

impl RemoteObject {
    pub fn new_unchecked(platform: Platform, version: Option<String>, key: &str) -> Self {
        Self {
            platform,
            version,
            key: key.to_owned(),
        }
    }

    pub fn platform(&self) -> Platform {
        self.platform
    }

    pub fn version(&self) -> &Option<String> {
        &self.version
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn debuglogs_url(&self) -> String {
        match &self.version {
            Some(version) => format!(
                "{}{}/{}/{}{}",
                BASE_DEBUGLOGS_URL,
                self.platform.to_string().to_lowercase(),
                version,
                self.key,
                self.platform.debuglogs_url_ending()
            ),
            None => format!(
                "{}{}{}",
                BASE_DEBUGLOGS_URL,
                self.key,
                self.platform.debuglogs_url_ending()
            ),
        }
    }

    pub fn fetchable_url(&self) -> String {
        format!(
            "{}{}/{}{}",
            BASE_WORKER_URL,
            self.platform.to_string().to_lowercase(),
            self.key,
            match &self.version {
                Some(version) => format!("?v={}", version),
                None => "".to_string(),
            },
        )
    }
}

#[traceable_parser]
pub fn remote_object(input: Span) -> IResult<Span, RemoteObject> {
    map(
        preceded(
            tag(BASE_DEBUGLOGS_URL),
            verify(
                tuple((
                    opt(terminated(
                        separated_pair(
                            alt((
                                value(
                                    Platform::Android,
                                    tag(Platform::Android.to_string().to_lowercase().as_str()),
                                ),
                                value(
                                    Platform::Ios,
                                    tag(Platform::Ios.to_string().to_lowercase().as_str()),
                                ),
                                value(
                                    Platform::Desktop,
                                    tag(Platform::Desktop.to_string().to_lowercase().as_str()),
                                ),
                            )),
                            tag("/"),
                            is_not("/"),
                        ),
                        tag("/"),
                    )),
                    verify(is_a("abcdef1234567890"), |s: &Span| s.len() == KEY_LENGTH),
                    alt((
                        value(Platform::Ios, tag(Platform::Ios.debuglogs_url_ending())),
                        value(
                            Platform::Desktop,
                            tag(Platform::Desktop.debuglogs_url_ending()),
                        ),
                        value(Platform::Android, not(tag("."))),
                    )),
                )),
                |(opt, _, platform): &(_, Span, _)| match opt {
                    Some((explcit_platform, _)) => platform == explcit_platform,
                    None => true,
                },
            ),
        ),
        |(opt, key, platform): (_, Span, _)| match opt {
            Some((_, version)) => {
                RemoteObject::new_unchecked(platform, Some(version.to_string()), key.fragment())
            }
            None => RemoteObject::new_unchecked(platform, None, key.fragment()),
        },
    )(input)
}

impl_from_str!(remote_object => RemoteObject);

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;
    use crate::{test_parsing, test_parsing_err_or_remainder};

    #[test_case(
        "https://debuglogs.org/0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123",
        RemoteObject::new_unchecked(Platform::Android, None, "0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123");
        "android"
    )]
    #[test_case(
        "https://debuglogs.org/0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123.zip",
        RemoteObject::new_unchecked(Platform::Ios, None, "0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123");
        "ios"
    )]
    #[test_case(
        "https://debuglogs.org/0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123.gz",
        RemoteObject::new_unchecked(Platform::Desktop, None, "0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123");
        "desktop"
    )]
    fn parsing_old_ok(input: &str, output: RemoteObject) {
        test_parsing(remote_object, input, "", output);
    }

    #[test_case(
        "https://debuglogs.org/android/1.23.4/0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123",
        RemoteObject::new_unchecked(Platform::Android, Some("1.23.4".to_owned()), "0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123");
        "android"
    )]
    #[test_case(
        "https://debuglogs.org/ios/1.23.4/0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123.zip",
        RemoteObject::new_unchecked(Platform::Ios, Some("1.23.4".to_owned()), "0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123");
        "ios"
    )]
    #[test_case(
        "https://debuglogs.org/desktop/1.23.4/0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123.gz",
        RemoteObject::new_unchecked(Platform::Desktop, Some("1.23.4".to_owned()), "0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123");
        "desktop"
    )]
    fn parsing_new_ok(input: &str, output: RemoteObject) {
        test_parsing(remote_object, input, "", output);
    }

    #[test_case(
        "https://debuglogs.org/123/0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123";
        "no explicit platform but has version"
    )]
    #[test_case(
        "https://debuglogs.org/android/0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123";
        "no version but has explicit platform"
    )]
    #[test_case(
        "https://debuglogs.org/unknown/123/0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123";
        "unknown explicit platform"
    )]
    #[test_case(
        "https://debuglogs.org/ios/123/0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123";
        "explicit platform doesn't match platform"
    )]
    #[test_case(
        "https://debuglogs.org/0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123.invalid";
        "invalid extension"
    )]
    #[test_case(
        "https://debuglogs.org/0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd012.gz";
        "invalid length"
    )]
    #[test_case(
        "https://debuglogs.org/012345#789abcdefg@ij0123456789%bcdefabcd&12345!789ab?defabcd012.zip";
        "non-alphanumeric characters in key"
    )]
    #[test_case(
        "0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123.gz";
        "alphanumeric but invalid characters in key"
    )]
    #[test_case(
        "https://debuglogs.org/0123456789abcdefabcd0123456789/abcdefabcd0123456789abcdefabcd0123.zip";
        "invalid length because of path segments 1"
    )]
    #[test_case(
        "https://debuglogs.org/abcdefabcd0123456789abcdefabcd0123/.zip";
        "invalid length because of path segments 2"
    )]
    #[test_case(
        "0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123.gz";
        "no beginning"
    )]
    fn parsing_err(input: &str) {
        test_parsing_err_or_remainder(remote_object, input);
    }

    #[test_case(
        RemoteObject::new_unchecked(Platform::Android, None, "0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123") =>
        "https://debuglogs.org/0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123".to_owned();
        "android"
    )]
    #[test_case(
        RemoteObject::new_unchecked(Platform::Ios, None, "0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123") =>
        "https://debuglogs.org/0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123.zip".to_owned();
        "ios"
    )]
    #[test_case(
        RemoteObject::new_unchecked(Platform::Desktop, None, "0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123") =>
        "https://debuglogs.org/0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123.gz".to_owned();
        "desktop"
    )]
    fn debuglogs_url_old(input: RemoteObject) -> String {
        input.debuglogs_url()
    }

    #[test_case(
        RemoteObject::new_unchecked(Platform::Android, Some("123".to_owned()), "0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123") =>
        "https://debuglogs.org/android/123/0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123".to_owned();
        "android"
    )]
    #[test_case(
        RemoteObject::new_unchecked(Platform::Ios, Some("123".to_owned()), "0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123") =>
        "https://debuglogs.org/ios/123/0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123.zip".to_owned();
        "ios"
    )]
    #[test_case(
        RemoteObject::new_unchecked(Platform::Desktop, Some("123".to_owned()), "0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123") =>
        "https://debuglogs.org/desktop/123/0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123.gz".to_owned();
        "desktop"
    )]
    fn debuglogs_url_new(input: RemoteObject) -> String {
        input.debuglogs_url()
    }

    #[test_case(
        RemoteObject::new_unchecked(Platform::Android, None, "0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123") =>
        "https://getlogs.warp.workers.dev/android/0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123".to_owned();
        "android"
    )]
    #[test_case(
        RemoteObject::new_unchecked(Platform::Ios, None, "0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123") =>
        "https://getlogs.warp.workers.dev/ios/0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123".to_owned();
        "ios"
    )]
    #[test_case(
        RemoteObject::new_unchecked(Platform::Desktop, None, "0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123") =>
        "https://getlogs.warp.workers.dev/desktop/0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123".to_owned();
        "desktop"
    )]
    fn fetchable_url_old(input: RemoteObject) -> String {
        input.fetchable_url()
    }

    #[test_case(
        RemoteObject::new_unchecked(Platform::Android, Some("123".to_owned()), "0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123") =>
        "https://getlogs.warp.workers.dev/android/0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123?v=123".to_owned();
        "android"
    )]
    #[test_case(
        RemoteObject::new_unchecked(Platform::Ios, Some("123".to_owned()), "0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123") =>
        "https://getlogs.warp.workers.dev/ios/0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123?v=123".to_owned();
        "ios"
    )]
    #[test_case(
        RemoteObject::new_unchecked(Platform::Desktop, Some("123".to_owned()), "0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123") =>
        "https://getlogs.warp.workers.dev/desktop/0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123?v=123".to_owned();
        "desktop"
    )]
    fn fetchable_url_new(input: RemoteObject) -> String {
        input.fetchable_url()
    }
}
