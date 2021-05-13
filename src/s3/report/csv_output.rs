// CsvOutput
use crate::s3::{
    BucketAcl,
    BucketEncryption,
    BucketLogging,
    BucketWebsite,
    MfaStatus,
    PublicAccessBlockType,
    VersioningStatus,
};
use serde::Serialize;
use super::Report;

#[derive(Default, Serialize)]
pub struct CsvOutput {
    name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    acl: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    block_public_acls: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    block_public_policy: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    encryption: Option<Option<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    ignore_public_acls: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    logging: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    mfa_delete: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    policy_wildcard_principals: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    restrict_public_buckets: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    versioning: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    website: Option<bool>,
}

impl From<&Report> for CsvOutput {
    fn from(report: &Report) -> Self {
        // Initial CSV output with just a name and defaults
        let mut output = Self {
            name: report.name.to_string(),
            ..Default::default()
        };

        // ACL
        output.acl = if let Some(acl) = &report.acl {
            let acl = match &acl {
                BucketAcl::Private => "private".into(),
                BucketAcl::Public  => "public".into(),
            };

            Some(acl)
        }
        else {
            None
        };

        // Encryption
        output.encryption = if let Some(encryption) = &report.encryption {
            let encryption = match &encryption {
                BucketEncryption::Default    => Some("AES256".into()),
                BucketEncryption::Kms        => Some("aws:kms".into()),
                BucketEncryption::None       => Some("None".into()),
                BucketEncryption::Unknown(s) => Some(s.into()),
            };

            Some(encryption)
        }
        else {
            None
        };

        // Logging
        output.logging = if let Some(logging) = &report.logging {
            let logging = matches!(&logging, BucketLogging::Enabled(_));
            Some(logging)
        }
        else {
            None
        };

        // MFA Delete
        output.mfa_delete = if let Some(versioning) = &report.versioning {
            let mfa_delete = matches!(
                versioning.mfa_delete(),
                MfaStatus::Enabled,
            );

            Some(mfa_delete)
        }
        else {
            None
        };

        // Policy wildcards
        output.policy_wildcard_principals = if let Some(policy) = &report.policy {
            let policy = match &policy {
                None         => false,
                Some(policy) => policy.wildcards().count() > 0,
            };

            Some(policy)
        }
        else {
            None
        };

        // Public access blocks
        if let Some(blocks) = &report.public_access_block {
            for block in blocks.iter() {
                match block {
                    PublicAccessBlockType::BlockPublicAcls(b) => {
                        output.block_public_acls = Some(*b)
                    },
                    PublicAccessBlockType::BlockPublicPolicy(b) => {
                        output.block_public_policy = Some(*b)
                    },
                    PublicAccessBlockType::IgnorePublicAcls(b) => {
                        output.ignore_public_acls = Some(*b)
                    },
                    PublicAccessBlockType::RestrictPublicBuckets(b) => {
                        output.restrict_public_buckets = Some(*b)
                    },
                }
            }
        }
        else {
            output.block_public_acls       = None;
            output.block_public_policy     = None;
            output.ignore_public_acls      = None;
            output.restrict_public_buckets = None;
        }

        // Versioning
        output.versioning = if let Some(versioning) = &report.versioning {
            let versioning = matches!(
                versioning.versioning(),
                VersioningStatus::Enabled,
            );

            Some(versioning)
        }
        else {
            None
        };

        // Website
        output.website = if let Some(website) = &report.website {
            let website = matches!(&website, BucketWebsite::Enabled);
            Some(website)
        }
        else {
            None
        };

        output
    }
}
