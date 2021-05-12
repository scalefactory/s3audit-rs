// Bucket reporting in various formats
use anyhow::Result;
use colored::*;
use crate::common::Emoji;
use crate::s3::{
    BucketAcl,
    BucketEncryption,
    BucketLogging,
    BucketPolicy,
    BucketVersioning,
    BucketWebsite,
    MfaStatus,
    NoBucketPolicy,
    PublicAccessBlock,
    PublicAccessBlockType,
    VersioningStatus,
};
use serde::Serialize;
use std::io;
use std::str::FromStr;

#[derive(Debug)]
pub enum ReportType {
    Csv,
    Text,
}

impl Default for ReportType {
    fn default() -> Self {
        Self::Text
    }
}

impl FromStr for ReportType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();

        match s.as_str() {
            "csv"  => Ok(Self::Csv),
            "text" => Ok(Self::Text),
            _      => Err(anyhow::anyhow!("Unknown Report Type")),
        }
    }
}

#[derive(Debug, Default)]
pub struct ReportOptions {
    pub output_type: ReportType,
}

#[derive(Debug)]
pub struct Report {
    pub name:                String,
    pub acl:                 Option<BucketAcl>,
    pub encryption:          Option<BucketEncryption>,
    pub logging:             Option<BucketLogging>,
    pub policy:              Option<Option<BucketPolicy>>,
    pub public_access_block: Option<PublicAccessBlock>,
    pub versioning:          Option<BucketVersioning>,
    pub website:             Option<BucketWebsite>,
}

#[derive(Default, Serialize)]
struct CsvOutput {
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

impl Report {
    pub fn output(&self, options: &ReportOptions) -> Result<()> {
        match options.output_type {
            ReportType::Csv  => self.csv(),
            ReportType::Text => {
                self.text();
                Ok(())
            },
        }
    }

    // CSV output
    pub fn csv(&self) -> Result<()> {
        let mut writer = csv::Writer::from_writer(io::stdout());
        let output: CsvOutput = self.into();
        writer.serialize(output)?;
        writer.flush()?;

        Ok(())
    }

    // Simple text output
    pub fn text(&self) {
        let name = self.name.bold().blue().to_string();

        println!("  {} {}", Emoji::Arrow, &name);

        // Public access configuration
        if let Some(blocks) = &self.public_access_block {
            println!("    {} Bucket public access configuration", Emoji::Arrow);

            for block in blocks.iter() {
                println!("      {}", block);
            }
        }

        // Encryption
        if let Some(encryption) = &self.encryption {
            println!("    {}", encryption);
        }

        // Versioning and MFA Delete
        if let Some(versioning) = &self.versioning {
            println!("    {}", versioning.versioning());
            println!("    {}", versioning.mfa_delete());
        }

        // Static website hosting
        if let Some(website) = &self.website {
            println!("    {}", website);
        }

        // Bucket policy
        if let Some(policy) = &self.policy {
            match &policy {
                None => {
                    println!("    {}", NoBucketPolicy { })
                },
                Some(policy) => {
                    println!("    {}", policy.wildcards());
                    println!("    {}", policy.cloudfront_distributions());
                },
            }
        }

        // Bucket ACL
        if let Some(acl) = &self.acl {
            println!("    {}", acl);
        }

        // Bucket logging
        if let Some(logging) = &self.logging {
            println!("    {}", logging);
        }
    }
}
