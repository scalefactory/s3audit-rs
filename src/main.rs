//! s3audit-rs: A tool for auditing S3 buckets
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::redundant_field_names)]
use anyhow::Result;
use atty::Stream;
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
    )]
    disable_check: Option<Vec<Audit>>,

    /// Enable specific audits
    #[structopt(
        long,
        short,
        value_name = "AUDIT",
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

fn should_colour_output() -> bool {
    if !atty::is(Stream::Stdout) {
        // STDOUT is not a pseudoterminal
        return false;
    }

    // Respect NO_COLOR environment variable
    // https://no-color.org/
    // If the variable is present, disable colour regardless of the value
    if env::var("NO_COLOR").is_ok() {
        return false;
    }

    match env::var("TERM") {
        Err(_) => {
            // Not sure about terminal type; play safe
            false
        },
        Ok(termtype) => {
            // Use colour unless dumb terminal detected
            termtype != "dumb"
        },
    }
}

#[tokio::main]
async fn main() -> Result<()> {
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
        coloured:    should_colour_output(),
        output_type: cli.format,
    };

    for report in reports {
        report.output(&report_options)?;
    }

    Ok(())
}
