use derive_more::{Display, IsVariant};

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, IsVariant)]
pub enum Platform {
    Android,
    #[display(fmt = "iOS")]
    Ios,
    Desktop,
}

impl Platform {
    pub const fn debuglogs_url_ending(&self) -> &'static str {
        match &self {
            Platform::Android => "",
            Platform::Ios => ".zip",
            Platform::Desktop => ".gz",
        }
    }
}
