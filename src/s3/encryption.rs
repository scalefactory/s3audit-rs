// Bucket encryption config
use crate::common::Emoji;
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::get_bucket_encryption::{
    GetBucketEncryptionError,
    GetBucketEncryptionOutput,
};
use aws_sdk_s3::types::ServerSideEncryption;
use std::fmt;

#[derive(Debug, Eq, PartialEq)]
pub enum BucketEncryption {
    Default,
    Kms,
    None,
    Unknown(String),
}

// Type alias to avoid long lines in From impl.
type EncryptionResult = Result<
    GetBucketEncryptionOutput,
    SdkError<GetBucketEncryptionError>,
>;

// Could probably replace a log of this with some .and_then shenanigans.
impl From<GetBucketEncryptionOutput> for BucketEncryption {
    fn from(output: GetBucketEncryptionOutput) -> Self {
        let sse_algorithm = output.server_side_encryption_configuration
            .and_then(|config| config.rules)
            .and_then(|rules| {
                if rules.is_empty() {
                    None
                }
                else {
                    // first() returns an Option<&T>, we need an Option<T>
                    rules.first().cloned()
                }
            })
            .and_then(|rule| rule.apply_server_side_encryption_by_default)
            .and_then(|rule| rule.sse_algorithm);

        match sse_algorithm {
            None                               => Self::None,
            Some(ServerSideEncryption::Aes256) => Self::Default,
            Some(ServerSideEncryption::AwsKms) => Self::Kms,
            Some(unknown)                      => {
                Self::Unknown(unknown.as_str().into())
            },
        }
    }
}

impl From<EncryptionResult> for BucketEncryption {
    fn from(res: EncryptionResult) -> Self {
        match res {
            Ok(output) => Self::from(output),
            Err(_)     => Self::None,
        }
    }
}

impl fmt::Display for BucketEncryption {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = match self {
            Self::Default => {
                format!(
                    "{} Server side encryption enabled using the default AES256 algorithm",
                    Emoji::Info,
                )
            },
            Self::Kms => {
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

#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_s3::types::{
        ServerSideEncryptionByDefault,
        ServerSideEncryptionConfiguration,
        ServerSideEncryptionRule,
    };

    #[test]
    fn test_from_default_encryption() {
        let default = ServerSideEncryptionByDefault::builder()
            .sse_algorithm(ServerSideEncryption::Aes256)
            .build();

        let rule = ServerSideEncryptionRule::builder()
            .apply_server_side_encryption_by_default(default)
            .build();

        let configuration = ServerSideEncryptionConfiguration::builder()
            .rules(rule)
            .build();

        let output = GetBucketEncryptionOutput::builder()
            .server_side_encryption_configuration(configuration)
            .build();

        let expected = BucketEncryption::Default;

        let bucket_encryption: BucketEncryption = output.into();

        assert_eq!(bucket_encryption, expected)
    }

    #[test]
    fn test_from_kms_encryption() {
        let default = ServerSideEncryptionByDefault::builder()
            .kms_master_key_id("arn:aws:foo:bar:test")
            .sse_algorithm(ServerSideEncryption::AwsKms)
            .build();

        let rule = ServerSideEncryptionRule::builder()
            .apply_server_side_encryption_by_default(default)
            .build();

        let configuration = ServerSideEncryptionConfiguration::builder()
            .rules(rule)
            .build();

        let output = GetBucketEncryptionOutput::builder()
            .server_side_encryption_configuration(configuration)
            .build();

        let expected = BucketEncryption::Kms;

        let bucket_encryption: BucketEncryption = output.into();

        assert_eq!(bucket_encryption, expected);
    }

    #[test]
    fn test_from_unknown_encryption() {
        let default = ServerSideEncryptionByDefault::builder()
            .sse_algorithm(ServerSideEncryption::from("wat"))
            .build();

        let rule = ServerSideEncryptionRule::builder()
            .apply_server_side_encryption_by_default(default)
            .build();

        let configuration = ServerSideEncryptionConfiguration::builder()
            .rules(rule)
            .build();

        let output = GetBucketEncryptionOutput::builder()
            .server_side_encryption_configuration(configuration)
            .build();

        let expected = BucketEncryption::Unknown("wat".into());

        let bucket_encryption: BucketEncryption = output.into();

        assert_eq!(bucket_encryption, expected);
    }

    #[test]
    fn test_from_no_rules() {
        let configuration = ServerSideEncryptionConfiguration::builder()
            .set_rules(Some(Vec::new()))
            .build();

        let output = GetBucketEncryptionOutput::builder()
            .server_side_encryption_configuration(configuration)
            .build();

        let expected = BucketEncryption::None;

        let bucket_encryption: BucketEncryption = output.into();

        assert_eq!(bucket_encryption, expected);
    }

    #[test]
    fn test_from_no_sse_config() {
        let output = GetBucketEncryptionOutput::builder()
            .set_server_side_encryption_configuration(None)
            .build();

        let expected = BucketEncryption::None;

        let bucket_encryption: BucketEncryption = output.into();

        assert_eq!(bucket_encryption, expected);
    }
}
