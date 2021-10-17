use crate::{impl_from_str, Platform};

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

impl_from_str!(crate::parsers::remote_object => RemoteObject);

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

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
