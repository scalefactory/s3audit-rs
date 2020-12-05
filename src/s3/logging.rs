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
        output.logging_enabled.map_or(Self::Disabled, |logging| {
            Self::Enabled(logging.target_bucket)
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    use rusoto_s3::LoggingEnabled;

    #[test]
    fn test_from_for_bucket_logging_enabled() {
        let logging_enabled = LoggingEnabled {
            target_bucket: "foo".into(),
            ..Default::default()
        };

        let output = GetBucketLoggingOutput {
            logging_enabled: Some(logging_enabled),
        };

        let expected = BucketLogging::Enabled("foo".into());

        let logging: BucketLogging = output.into();

        assert_eq!(logging, expected)
    }

    #[test]
    fn test_from_for_bucket_logging_disabled() {
        let output = GetBucketLoggingOutput {
            logging_enabled: None,
        };

        let expected = BucketLogging::Disabled;

        let logging: BucketLogging = output.into();

        assert_eq!(logging, expected)
    }
}
