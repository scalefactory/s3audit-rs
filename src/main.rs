//! s3audit-rs: A tool for auditing S3 buckets
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::redundant_field_names)]
use anyhow::Result;

mod common;
mod s3;

#[tokio::main]
async fn main() -> Result<()> {
    let client = s3::Client::new();

    client.report_all().await?;

    Ok(())
}
