// Bucket website
use crate::common::Emoji;
use rusoto_core::RusotoError;
use rusoto_s3::{
    GetBucketWebsiteError,
    GetBucketWebsiteOutput,
};
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum BucketWebsite {
    Enabled,
    Disabled,
}

// Type alias to avoid long line in the From impl
type WebsiteResult = Result<GetBucketWebsiteOutput, RusotoError<GetBucketWebsiteError>>;

impl From<WebsiteResult> for BucketWebsite {
    fn from(res: WebsiteResult) -> Self {
        match res {
            Ok(_)  => Self::Enabled,
            Err(_) => Self::Disabled,
        }
    }
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
