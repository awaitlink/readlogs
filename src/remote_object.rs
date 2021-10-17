use nom::{
    branch::alt,
    bytes::complete::{is_a, tag},
    combinator::{map, not, value, verify},
    sequence::{pair, preceded},
    IResult,
};

use crate::{impl_from_str, Platform};

pub const KEY_LENGTH: usize = 64;
pub const BASE_WORKER_URL: &str = "https://getlogs.warp.workers.dev/";

#[derive(Debug, Clone, PartialEq)]
pub struct RemoteObject {
    platform: Platform,
    key: String,
}

impl RemoteObject {
    pub fn new_unchecked(platform: Platform, key: &str) -> Self {
        Self {
            platform,
            key: key.to_owned(),
        }
    }

    pub fn platform(&self) -> Platform {
        self.platform
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn debuglogs_url(&self) -> String {
        "https://debuglogs.org/".to_owned() + &self.key + self.platform.debuglogs_url_ending()
    }

    pub fn fetchable_url(&self) -> String {
        BASE_WORKER_URL.to_owned() + &self.platform.to_string().to_lowercase() + "/" + &self.key
    }
}

pub fn remote_object(input: &str) -> IResult<&str, RemoteObject> {
    map(
        preceded(
            tag("https://debuglogs.org/"),
            pair(
                verify(is_a("abcdef1234567890"), |s: &str| s.len() == KEY_LENGTH),
                alt((
                    value(Platform::Ios, tag(Platform::Ios.debuglogs_url_ending())),
                    value(
                        Platform::Desktop,
                        tag(Platform::Desktop.debuglogs_url_ending()),
                    ),
                    value(Platform::Android, not(tag("."))),
                )),
            ),
        ),
        |(key, platform): (&str, Platform)| RemoteObject::new_unchecked(platform, key),
    )(input)
}

impl_from_str!(remote_object => RemoteObject);

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    use crate::parsing_test;

    #[test_case(
        "https://debuglogs.org/0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123" =>
        RemoteObject::new_unchecked(
            Platform::Android,
            "0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123"
        );
        "android"
    )]
    #[test_case(
        "https://debuglogs.org/0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123.zip" =>
        RemoteObject::new_unchecked(
            Platform::Ios,
            "0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123"
        );
        "ios"
    )]
    #[test_case(
        "https://debuglogs.org/0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123.gz" =>
        RemoteObject::new_unchecked(
            Platform::Desktop,
            "0123456789abcdefabcd0123456789abcdefabcd0123456789abcdefabcd0123"
        );
        "desktop"
    )]
    fn parsing_ok(input: &str) -> RemoteObject {
        parsing_test(remote_object, input)
    }

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
        assert!(remote_object(input).is_err());
    }

    #[test_case(
        RemoteObject::new_unchecked(
            Platform::Android,
            "0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123"
        ) =>
        "https://debuglogs.org/0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123".to_owned();
        "android"
    )]
    #[test_case(
        RemoteObject::new_unchecked(
            Platform::Ios,
            "0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123"
        ) =>
        "https://debuglogs.org/0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123.zip".to_owned();
        "ios"
    )]
    #[test_case(
        RemoteObject::new_unchecked(
            Platform::Desktop,
            "0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123"
        ) =>
        "https://debuglogs.org/0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123.gz".to_owned();
        "desktop"
    )]
    fn debuglogs_url(input: RemoteObject) -> String {
        input.debuglogs_url()
    }

    #[test_case(
        RemoteObject::new_unchecked(
            Platform::Android,
            "0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123"
        ) =>
        BASE_WORKER_URL.to_owned() + "android/0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123";
        "android"
    )]
    #[test_case(
        RemoteObject::new_unchecked(
            Platform::Ios,
            "0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123"
        ) =>
        BASE_WORKER_URL.to_owned() + "ios/0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123";
        "ios"
    )]
    #[test_case(
        RemoteObject::new_unchecked(
            Platform::Desktop,
            "0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123"
        ) =>
        BASE_WORKER_URL.to_owned() + "desktop/0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij0123";
        "desktop"
    )]
    fn fetchable_url(input: RemoteObject) -> String {
        input.fetchable_url()
    }
}
