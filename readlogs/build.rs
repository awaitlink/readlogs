use anyhow::Result;
use vergen::{vergen, Config, ShaKind, TimestampKind};

fn main() -> Result<()> {
    let mut config = Config::default();

    *config.git_mut().sha_kind_mut() = ShaKind::Short;

    *config.build_mut().enabled_mut() = true;
    *config.build_mut().kind_mut() = TimestampKind::DateOnly;

    vergen(config)
}
