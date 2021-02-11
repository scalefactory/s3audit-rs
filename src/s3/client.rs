// S3 client implementation
use crate::common::Emoji;
use crate::s3::{
    acl::BucketAcl,
    encryption::BucketEncryption,
    logging::BucketLogging,
    policy::BucketPolicy,
    policy::NoBucketPolicy,
    public_access_block::PublicAccessBlock,
    versioning::BucketVersioning,
    website::BucketWebsite,
};
use anyhow::Result;
use atty::Stream;
use colored::*;
use rusoto_core::{Region, RusotoError};
use rusoto_s3::{
    GetBucketAclRequest,
    GetBucketEncryptionRequest,
    GetBucketLoggingRequest,
    GetBucketPolicyRequest,
    GetBucketVersioningRequest,
    GetBucketWebsiteRequest,
    GetPublicAccessBlockRequest,
    S3,
    S3Client,
};
use std::convert::TryInto;
use std::env;

pub struct Client {
    client: S3Client,
}

#[derive(Default)]
pub struct ReportOptions {
    coloured_output: bool,
}

impl Client {
    // Get a new S3 client
    pub fn new() -> Self {
        let region = Region::default();
        let client = S3Client::new(region);

        Self {
            client: client,
        }
    }

    // List all buckets on an account
    async fn list_buckets(&self) -> Result<Vec<String>> {
        let output = self.client.list_buckets().await?;

        let bucket_names = output.buckets.map_or_else(Vec::new, |buckets| {
            buckets.iter()
                .filter_map(|bucket| bucket.name.to_owned())
                .collect()
        });

        Ok(bucket_names)
    }

    async fn get_bucket_acl(&self, bucket: &str) -> Result<BucketAcl> {
        let input = GetBucketAclRequest {
            bucket: bucket.into(),
            ..Default::default()
        };

        let output = self.client.get_bucket_acl(input).await?;
        let config: BucketAcl = output.into();

        Ok(config)
    }

    async fn get_bucket_encryption(&self, bucket: &str) -> Result<BucketEncryption> {
        let input = GetBucketEncryptionRequest {
            bucket: bucket.to_owned(),
            ..Default::default()
        };

        let output = self.client.get_bucket_encryption(input).await;
        let config: BucketEncryption = output.into();

        Ok(config)
    }

    async fn get_bucket_logging(&self, bucket: &str) -> Result<BucketLogging> {
        let input = GetBucketLoggingRequest {
            bucket: bucket.into(),
            ..Default::default()
        };

        let output = self.client.get_bucket_logging(input).await?;
        let config: BucketLogging = output.into();

        Ok(config)
    }

    async fn get_bucket_policy(&self, bucket: &str) -> Result<Option<BucketPolicy>> {
        let input = GetBucketPolicyRequest {
            bucket: bucket.into(),
            ..Default::default()
        };

        let output = match self.client.get_bucket_policy(input).await {
            Ok(item) => Ok(item),
            Err(RusotoError::Unknown(response)) => {
                if response.status == 404 {
                    return Ok(None) // there's no bucket policy; not an error
                } else {
                    Err(RusotoError::Unknown(response))
                }
            }
            Err(error) => {
                Err(error)
            }
        }?;

        // Didn't get 404 but no policy supplied
        if output.policy.is_none() {
            return Ok(None);
        }

        let bucket_policy = TryInto::<BucketPolicy>::try_into(output)?;
        Ok(Some(bucket_policy))
    }

    async fn get_bucket_versioning(&self, bucket: &str) -> Result<BucketVersioning> {
        let input = GetBucketVersioningRequest {
            bucket: bucket.into(),
            ..Default::default()
        };

        let output = self.client.get_bucket_versioning(input).await?;
        let config: BucketVersioning = output.into();

        Ok(config)
    }

    async fn get_bucket_website(&self, bucket: &str) -> Result<BucketWebsite> {
        let input = GetBucketWebsiteRequest {
            bucket: bucket.into(),
            ..Default::default()
        };

        // Note that we aren't using the `?` operator here.
        let output = self.client.get_bucket_website(input).await;
        let config: BucketWebsite = output.into();

        Ok(config)
    }

    // Get the bucket's public access block configuration
    async fn get_public_access_block(&self, bucket: &str) -> Result<PublicAccessBlock> {
        let input = GetPublicAccessBlockRequest {
            bucket: bucket.to_owned(),
            ..Default::default()
        };

        let output = self.client.get_public_access_block(input).await?;
        let config: PublicAccessBlock = output.into();

        Ok(config)
    }

    // Reports on a single bucket
    pub async fn report(&self, bucket: &str, options: &ReportOptions) -> Result<()> {

        // Highlight bucket name if appropriate
        let bucket_name_optionally_coloured = match options.coloured_output {
            true => bucket.bold().blue().to_string(),
            _ => bucket.to_string(),
        };

        println!("  {} {}", Emoji::Arrow, &bucket_name_optionally_coloured);

        // Public access configuration
        println!("    {} Bucket public access configuration", Emoji::Arrow);

        let public_access_blocks = self.get_public_access_block(bucket).await?;
        for block in public_access_blocks.iter() {
            println!("      {}", block);
        }

        // Bucket encryption
        let bucket_encryption = self.get_bucket_encryption(bucket).await?;
        println!("    {}", bucket_encryption);

        // Bucket versioning and MFA delete
        let bucket_versioning = self.get_bucket_versioning(bucket).await?;
        println!("    {}", bucket_versioning.versioning());
        println!("    {}", bucket_versioning.mfa_delete());

        // Static website hosting
        let bucket_website = self.get_bucket_website(bucket).await?;
        println!("    {}", bucket_website);

        // Bucket policy
        match self.get_bucket_policy(bucket).await? {
            None => {
                println!("    {}", NoBucketPolicy { });
            },
            Some(policy) => {
                println!("    {}", policy.wildcards());
                println!("    {}", policy.cloudfront_distributions());
            }
        }

        // Bucket ACL
        let bucket_acl = self.get_bucket_acl(bucket).await?;
        println!("    {}", bucket_acl);

        // Bucket logging
        let bucket_logging = self.get_bucket_logging(bucket).await?;
        println!("    {}", bucket_logging);

        Ok(())
    }

    fn should_colour_output(&self) -> bool {
        if !atty::is(Stream::Stdout) {
            // STDOUT is not a pseudoterminal
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


    // Reports on all discovered buckets
    pub async fn report_all(&self) -> Result<()> {
        let buckets = self.list_buckets().await?;

        let options = ReportOptions {
            // Use coloured output
            coloured_output: self.should_colour_output(),
        };

        for bucket in buckets.iter() {
            self.report(&bucket, &options).await?;
        }

        Ok(())
    }
}
