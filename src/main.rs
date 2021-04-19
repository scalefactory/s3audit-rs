//! s3audit-rs: A tool for auditing S3 buckets
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::redundant_field_names)]
use anyhow::Result;
use atty::Stream;
use clap::{
    crate_description,
    crate_name,
    crate_version,
    App,
    Arg,
};
use std::env;

mod common;
mod s3;

use s3::{
    ReportOptions,
    ReportType,
};

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
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .arg(
            Arg::from_usage("-p, --profile=[NAME] 'Specify an AWS profile name to use'")
        )
        .get_matches();

    if let Some(profile_name) = matches.value_of("profile") {
        env::set_var("AWS_PROFILE", &*profile_name);
    }

    let client = s3::Client::new();
    let reports = client.report_all().await?;

    let report_options = ReportOptions {
        coloured:    should_colour_output(),
        output_type: ReportType::Text,
    };

    for report in reports {
        report.output(&report_options)?;
    }

    Ok(())
}
