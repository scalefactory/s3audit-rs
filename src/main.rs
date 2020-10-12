//! s3audit-rs: A tool for auditing S3 buckets
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::redundant_field_names)]
use anyhow::Result;

mod common;
mod s3;

#[tokio::main]
async fn main() -> Result<()> {
    let app = clap::App::new(clap::crate_name!())
      .version(clap::crate_version!())
      .about(clap::crate_description!())
      .arg(clap::Arg::from_usage("-p, --profile=[NAME] 'Provides an input file to the program'"));

    if let Some(profile_name) = app.get_matches().value_of("profile") {
        std::env::set_var("AWS_PROFILE",&*profile_name);
    }

    let client = s3::Client::new();

    client.report_all().await?;

    Ok(())
}
