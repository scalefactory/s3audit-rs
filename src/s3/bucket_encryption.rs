// Bucket encryption config
use crate::common::Emoji;
use rusoto_core::RusotoError;
use rusoto_s3::{
    GetBucketEncryptionError,
    GetBucketEncryptionOutput,
};
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum BucketEncryption {
    Default,
    KMS,
    None,
    Unknown(String),
}

// Type alias to avoid long lines in From impl.
type EncryptionResult = Result<GetBucketEncryptionOutput, RusotoError<GetBucketEncryptionError>>;

// Could probably replace a log of this with some .and_then shenanigans.
impl From<GetBucketEncryptionOutput> for BucketEncryption {
    fn from(output: GetBucketEncryptionOutput) -> Self {
        let sse_algorithm = output.server_side_encryption_configuration
            .map(|config| config.rules)
            .and_then(|rules| {
                if rules.is_empty() {
                    None
                }
                else {
                    // first() returns an Option<&T>, we need an Option<T>
                    rules.first().map(|rule| rule.to_owned())
                }
            })
            .and_then(|rule| rule.apply_server_side_encryption_by_default)
            .map(|rule| rule.sse_algorithm);

        match sse_algorithm.as_deref() {
            Some("AES256")  => Self::Default,
            Some("aws:kms") => Self::KMS,
            Some(algorithm) => Self::Unknown(algorithm.into()),
            None            => Self::None,
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

#[cfg(test)]
mod tests {
    use super::*;
    use rusoto_s3::{
        ServerSideEncryptionByDefault,
        ServerSideEncryptionConfiguration,
        ServerSideEncryptionRule,
    };

    #[test]
    fn test_from_default_encryption() {
        let server_side_encryption_by_default = ServerSideEncryptionByDefault {
            sse_algorithm: "AES256".into(),
            ..Default::default()
        };

        let server_side_encryption_rule = ServerSideEncryptionRule {
            apply_server_side_encryption_by_default: Some(server_side_encryption_by_default),
        };

        let server_side_encryption_configuration = ServerSideEncryptionConfiguration {
            rules: vec![server_side_encryption_rule],
        };

        let output = GetBucketEncryptionOutput {
            server_side_encryption_configuration: Some(server_side_encryption_configuration),
        };

        let expected = BucketEncryption::Default;

        let bucket_encryption: BucketEncryption = output.into();

        assert_eq!(bucket_encryption, expected)
    }

    #[test]
    fn test_from_kms_encryption() {
        let server_side_encryption_by_default = ServerSideEncryptionByDefault {
            kms_master_key_id: Some("arn:aws:foo:bar:test".into()),
            sse_algorithm: "aws:kms".into(),
        };

        let server_side_encryption_rule = ServerSideEncryptionRule {
            apply_server_side_encryption_by_default: Some(server_side_encryption_by_default),
        };

        let server_side_encryption_configuration = ServerSideEncryptionConfiguration {
            rules: vec![server_side_encryption_rule],
        };

        let output = GetBucketEncryptionOutput {
            server_side_encryption_configuration: Some(server_side_encryption_configuration),
        };

        let expected = BucketEncryption::KMS;

        let bucket_encryption: BucketEncryption = output.into();

        assert_eq!(bucket_encryption, expected);
    }

    #[test]
    fn test_from_unknown_encryption() {
        let server_side_encryption_by_default = ServerSideEncryptionByDefault {
            sse_algorithm: "wat".into(),
            ..Default::default()
        };

        let server_side_encryption_rule = ServerSideEncryptionRule {
            apply_server_side_encryption_by_default: Some(server_side_encryption_by_default),
        };

        let server_side_encryption_configuration = ServerSideEncryptionConfiguration {
            rules: vec![server_side_encryption_rule],
        };

        let output = GetBucketEncryptionOutput {
            server_side_encryption_configuration: Some(server_side_encryption_configuration),
        };

        let expected = BucketEncryption::Unknown("wat".into());

        let bucket_encryption: BucketEncryption = output.into();

        assert_eq!(bucket_encryption, expected);
    }

    #[test]
    fn test_from_no_rules() {
        let server_side_encryption_configuration = ServerSideEncryptionConfiguration {
            rules: Vec::new()
        };

        let output = GetBucketEncryptionOutput {
            server_side_encryption_configuration: Some(server_side_encryption_configuration),
        };

        let expected = BucketEncryption::None;

        let bucket_encryption: BucketEncryption = output.into();

        assert_eq!(bucket_encryption, expected);
    }

    #[test]
    fn test_from_no_sse_config() {
        let output = GetBucketEncryptionOutput {
            server_side_encryption_configuration: None,
        };

        let expected = BucketEncryption::None;

        let bucket_encryption: BucketEncryption = output.into();

        assert_eq!(bucket_encryption, expected);
    }
}
