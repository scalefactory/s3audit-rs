// Bucket reporting in various formats
use colored::*;
use crate::common::Emoji;
use crate::s3::{
    BucketAcl,
    BucketEncryption,
    BucketLogging,
    BucketPolicy,
    BucketVersioning,
    BucketWebsite,
    NoBucketPolicy,
    PublicAccessBlock,
};

#[derive(Debug)]
pub enum ReportType {
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

impl Report {
    pub fn output(&self, options: &ReportOptions) {
        match options.output_type {
            ReportType::Text => {
                self.text(options.coloured)
            }
        }
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
