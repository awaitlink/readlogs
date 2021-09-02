use nom::{
    branch::alt,
    bytes::complete::{is_a, tag},
    combinator::{map, not, value, verify},
    sequence::{pair, preceded},
    IResult,
};

use crate::{Platform, RemoteObject};

pub const KEY_LENGTH: usize = 64;

pub fn remote_object(input: &str) -> IResult<&str, RemoteObject> {
    map(
        preceded(
            tag("https://debuglogs.org/"),
            pair(
                verify(is_a("abcdef1234567890"), |s: &str| s.len() == KEY_LENGTH),
                alt((
                    value(Platform::Ios, tag(".zip")),
                    value(Platform::Desktop, tag(".gz")),
                    value(Platform::Android, not(tag("."))),
                )),
            ),
        ),
        |(key, platform): (&str, Platform)| RemoteObject::new_unchecked(platform, key),
    )(input)
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use crate::parsing_test;

    use super::*;

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
    fn ok(input: &str) -> RemoteObject {
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
    fn err(input: &str) {
        assert!(remote_object(input).is_err());
    }
}
