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

// Type alias to avoid long long in From impl.
type EncryptionResult = Result<GetBucketEncryptionOutput, RusotoError<GetBucketEncryptionError>>;

// Could probably replace a log of this with some .and_then shenanigans.
impl From<GetBucketEncryptionOutput> for BucketEncryption {
    fn from(output: GetBucketEncryptionOutput) -> Self {
        // Get the rules if there are any
        let rules = if let Some(config) = output.server_side_encryption_configuration {
            config.rules
        }
        else {
            return Self::None;
        };

        // Only a single rule makes sense currently, try to get it.
        let rule = if !rules.is_empty() {
            // We should be guaranteed a rule here.
            rules.first().expect("first encryption rule")
        }
        else {
            return Self::None;
        };

        let rule = if let Some(rule) = &rule.apply_server_side_encryption_by_default {
            rule
        }
        else {
            return BucketEncryption::None;
        };

        match rule.sse_algorithm.as_ref() {
            "AES256"  => Self::Default,
            "aws:kms" => Self::KMS,
            algorithm => Self::Unknown(algorithm.into()),
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
