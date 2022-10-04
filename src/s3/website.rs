// Bucket website
use crate::common::Emoji;
use aws_sdk_s3::error::GetBucketWebsiteError;
use aws_sdk_s3::output::GetBucketWebsiteOutput;
use aws_sdk_s3::types::SdkError;
use std::fmt;

#[derive(Debug, Eq, PartialEq)]
pub enum BucketWebsite {
    Enabled,
    Disabled,
}

// Type alias to avoid long line in the From impl
type WebsiteResult = Result<GetBucketWebsiteOutput, SdkError<GetBucketWebsiteError>>;

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
