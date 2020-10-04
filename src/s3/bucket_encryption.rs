// Bucket encryption config
use crate::common::Emoji;
use rusoto_s3::GetBucketEncryptionOutput;
use std::fmt;

pub enum BucketEncryption {
    Default,
    KMS,
    None,
    Unknown(String),
}

// Could probably replace a log of this with some .and_then shenanigans.
impl From<GetBucketEncryptionOutput> for BucketEncryption {
    fn from(output: GetBucketEncryptionOutput) -> Self {
        // Get the rules if there are any
        let rules = if let Some(config) = output.server_side_encryption_configuration {
            config.rules
        }
        else {
            return BucketEncryption::None;
        };

        // Only a single rule makes sense currently, try to get it.
        let rule = if !rules.is_empty() {
            // We should be guaranteed a rule here.
            rules.first().expect("first encryption rule")
        }
        else {
            return BucketEncryption::None;
        };

        let rule = if let Some(rule) = &rule.apply_server_side_encryption_by_default {
            rule
        }
        else {
            return BucketEncryption::None;
        };

        match rule.sse_algorithm.as_ref() {
            "AES256"  => BucketEncryption::Default,
            "aws:kms" => BucketEncryption::KMS,
            algorithm => BucketEncryption::Unknown(algorithm.into()),
        }
    }
}

impl fmt::Display for BucketEncryption {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = match self {
            Self::Default => {
                format!(
                    "{} Server side encryption enabled using the default AES256 algorithm",
                    Emoji::Warning,
                )
            },
            Self::KMS => {
                format!(
                    "{} Server side encryption enabled using KMS",
                    Emoji::Tick,
                )
            },
            Self::None => {
                format!(
                    "{} Server side encryption is not enabled",
                    Emoji::Cross,
                )
            },
            Self::Unknown(algorithm) => {
                format!(
                    "{} Server side encryption using unknown algorithm: {}",
                    Emoji::Warning,
                    algorithm,
                )
            },
        };

        write!(f, "{}", output)
    }
}
