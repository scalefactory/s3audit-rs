// Bucket logging
use crate::common::Emoji;
use rusoto_s3::GetBucketLoggingOutput;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum BucketLogging {
    Enabled(String),
    Disabled,
}

impl From<GetBucketLoggingOutput> for BucketLogging {
    fn from(output: GetBucketLoggingOutput) -> Self {
        if let Some(logging_enabled) = output.logging_enabled {
            let target_bucket = logging_enabled.target_bucket.to_owned();

            Self::Enabled(target_bucket)
        }
        else {
            Self::Disabled
        }
    }
}

impl fmt::Display for BucketLogging {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = match self {
            Self::Enabled(bucket) => {
                let emoji = Emoji::Tick;
                format!("{} Logging to {}", emoji, bucket)
            },
            Self::Disabled => {
                let emoji = Emoji::Cross;
                format!("{} Logging is not enabled", emoji)
            }
        };

        write!(f, "{}", output)
    }
}
