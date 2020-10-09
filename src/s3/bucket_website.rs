// Bucket website
use crate::common::Emoji;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum BucketWebsite {
    Enabled,
    Disabled,
}

impl fmt::Display for BucketWebsite {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = match self {
            Self::Enabled => {
                let emoji = Emoji::Warning;
                format!("{} Static website hosting is enabled", emoji)
            },
            Self::Disabled => {
                let emoji = Emoji::Tick;
                format!("{} Static website hosting is disabled", emoji)
            },
        };

        write!(f, "{}", output)
    }
}
