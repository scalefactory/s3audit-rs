//! s3audit-rs: A tool for auditing S3 buckets
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::redundant_field_names)]
use anyhow::Result;
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

    client.report_all().await?;

    Ok(())
}
