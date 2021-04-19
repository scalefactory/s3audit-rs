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

#[derive(Debug, Default)]
pub struct ReportOptions {
    pub coloured: bool,
    pub output_type: ReportType,
}

#[derive(Debug)]
pub struct Report {
    pub name:                String,
    pub acl:                 BucketAcl,
    pub encryption:          BucketEncryption,
    pub logging:             BucketLogging,
    pub policy:              Option<BucketPolicy>,
    pub public_access_block: PublicAccessBlock,
    pub versioning:          BucketVersioning,
    pub website:             BucketWebsite,
}

#[derive(Default, Serialize)]
struct CsvOutput {
    name: String,
    acl: String,
    block_public_acls: bool,
    block_public_policy: bool,
    encryption: Option<String>,
    ignore_public_acls: bool,
    logging: bool,
    mfa_delete: bool,
    policy_wildcard_principals: bool,
    restrict_public_buckets: bool,
    versioning: bool,
    website: bool,
}

impl From<&Report> for CsvOutput {
    fn from(report: &Report) -> Self {
        // Initial CSV output with just a name and defaults
        let mut output = Self {
            name: report.name.to_string(),
            ..Default::default()
        };

        // ACL
        output.acl = match &report.acl {
            BucketAcl::Private => "private".into(),
            BucketAcl::Public  => "public".into(),
        };

        // Encryption
        output.encryption = match &report.encryption {
            BucketEncryption::Default    => Some("AES256".into()),
            BucketEncryption::Kms        => Some("aws:kms".into()),
            BucketEncryption::None       => Some("None".into()),
            BucketEncryption::Unknown(s) => Some(s.into()),
        };

        // Logging
        output.logging = matches!(&report.logging, BucketLogging::Enabled(_));

        // MFA Delete
        output.mfa_delete = matches!(
            report.versioning.mfa_delete(),
            MfaStatus::Enabled,
        );

        // Policy wildcards
        output.policy_wildcard_principals = match &report.policy {
            None         => false,
            Some(policy) => policy.wildcards().count() > 0,
        };

        // Public access blocks
        for block in report.public_access_block.iter() {
            match block {
                PublicAccessBlockType::BlockPublicAcls(b) => {
                    output.block_public_acls = *b
                },
                PublicAccessBlockType::BlockPublicPolicy(b) => {
                    output.block_public_policy = *b
                },
                PublicAccessBlockType::IgnorePublicAcls(b) => {
                    output.ignore_public_acls = *b
                },
                PublicAccessBlockType::RestrictPublicBuckets(b) => {
                    output.restrict_public_buckets = *b
                },
            }
        }

        // Versioning
        output.versioning = matches!(
            report.versioning.versioning(),
            VersioningStatus::Enabled,
        );

        // Website
        output.website = matches!(&report.website, BucketWebsite::Enabled);

        output
    }
}

impl Report {
    pub fn output(&self, options: &ReportOptions) -> Result<()> {
        match options.output_type {
            ReportType::Csv  => self.csv(),
            ReportType::Text => {
                self.text(options.coloured);
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
    pub fn text(&self, coloured: bool) {
        let name = match coloured {
            true => self.name.bold().blue().to_string(),
            _    => self.name.to_string(),
        };

        println!("  {} {}", Emoji::Arrow, &name);

        // Public access configuration
        println!("    {} Bucket public access configuration", Emoji::Arrow);

        for block in self.public_access_block.iter() {
            println!("      {}", block);
        }

        // Encryption
        println!("    {}", self.encryption);

        // Versioning and MFA Delete
        println!("    {}", self.versioning.versioning());
        println!("    {}", self.versioning.mfa_delete());

        // Static website hosting
        println!("    {}", self.website);

        // Bucket policy
        match &self.policy {
            None => {
                println!("    {}", NoBucketPolicy { })
            },
            Some(policy) => {
                println!("    {}", policy.wildcards());
                println!("    {}", policy.cloudfront_distributions());
            },
        }

        // Bucket ACL
        println!("    {}", self.acl);

        // Bucket logging
        println!("    {}", self.logging);
    }
}
