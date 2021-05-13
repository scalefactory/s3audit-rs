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
    NoBucketPolicy,
    PublicAccessBlock,
};
use std::io;

mod csv_output;
mod report_type;

pub use csv_output::*;
pub use report_type::*;

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

#[derive(Debug)]
pub struct Reports(Vec<Report>);

impl Report {
    // CSV output
    pub fn csv<W>(&self, writer: &mut csv::Writer<W>) -> Result<()>
    where W: ::std::io::Write,
    {
        let output: CsvOutput = self.into();
        writer.serialize(output)?;

        Ok(())
    }

    // Simple text output
    pub fn text(&self) {
        let name = self.name.bold().blue();

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

impl Reports {
    pub fn new(reports: Vec<Report>) -> Self {
        Self(reports)
    }

    pub fn output(&self, options: &ReportOptions) -> Result<()> {
        match options.output_type {
            ReportType::Csv  => self.csv()?,
            ReportType::Text => self.text(),
        }

        Ok(())
    }

    // CSV output
    // Wrapping the report CSV method and passing a writer here is necessary,
    // otherwise we end up with duplicate headers when dealing with multiple
    // buckets.
    pub fn csv(&self) -> Result<()> {
        let mut writer = csv::Writer::from_writer(io::stdout());

        for report in &self.0 {
            report.csv(&mut writer)?;
        }

        writer.flush()?;

        Ok(())
    }

    // Text output
    pub fn text(&self) {
        for report in &self.0 {
            report.text();
        }
    }
}
