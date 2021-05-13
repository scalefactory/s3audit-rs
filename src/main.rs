//! s3audit-rs: A tool for auditing S3 buckets
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::redundant_field_names)]
use anyhow::Result;
use colored::control::SHOULD_COLORIZE;
use std::env;
use structopt::StructOpt;

mod common;
mod s3;

use s3::{
    Audit,
    Audits,
    ReportOptions,
    ReportType,
};

#[derive(Debug, StructOpt)]
#[structopt(about, rename_all = "kebab")]
struct CliConfig {
    /// Specify a specific bucket to audit
    #[structopt(
        long,
        short,
        value_name = "BUCKET",
    )]
    bucket: Option<String>,

    /// Disable specific audits
    #[structopt(
        long,
        short,
        value_name = "AUDIT",
        possible_values = &[
            "acl",
            "all",
            "cloudfront",
            "encryption",
            "logging",
            "mfa",
            "mfa-delete",
            "policy",
            "public-access-blocks",
            "server-side-encryption",
            "sse",
            "versioning",
            "website",
        ],
    )]
    disable_check: Option<Vec<Audit>>,

    /// Enable specific audits
    #[structopt(
        long,
        short,
        value_name = "AUDIT",
        possible_values = &[
            "acl",
            "all",
            "cloudfront",
            "encryption",
            "logging",
            "mfa",
            "mfa-delete",
            "policy",
            "public-access-blocks",
            "server-side-encryption",
            "sse",
            "versioning",
            "website",
        ],
    )]
    enable_check: Option<Vec<Audit>>,

    /// Specify the report output format
    #[structopt(
        long,
        short,
        default_value = "text",
        possible_values = &["csv", "text"],
        value_name = "FORMAT",
    )]
    format: ReportType,

    /// Specify an AWS profile name to use
    #[structopt(
        long,
        short,
        value_name = "NAME",
    )]
    profile: Option<String>,
}

// The colored library does a lot of work for us here. It will check various
// environment variables, and ensure that we're outputting to stdout.
// All colorize methods will respect what happens here, so we should ONLY
// colour output via those methods.
fn should_colour_output() {
    // We also do a bit extra and ensure that the TERM is a decent type
    match env::var("TERM") {
        Err(_) => {
            // Not sure about terminal type; play safe
            SHOULD_COLORIZE.set_override(false)
        },
        Ok(termtype) => {
            // Use colour unless dumb terminal detected
            if termtype == "dumb" {
                SHOULD_COLORIZE.set_override(false)
            }
        },
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // A few extra checks on top of what colorize itself does.
    should_colour_output();

    let cli = CliConfig::from_args();

    // Set the AWS_PROFILE environment variable if the user requested a
    // specific profile.
    if let Some(profile_name) = cli.profile {
        env::set_var("AWS_PROFILE", &*profile_name);
    }

    // Work out which audits we're going to run.
    let audits = Audits::new()
        .disable(cli.disable_check)
        .enable(cli.enable_check)
        .enabled();

    let client = s3::Client::new();
    let reports = client.report(cli.bucket, audits).await?;

    let report_options = ReportOptions {
        output_type: cli.format,
    };

    reports.output(&report_options)?;

    Ok(())
}
