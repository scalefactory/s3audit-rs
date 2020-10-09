// S3 client implementation
use crate::common::Emoji;
use crate::s3::{
    bucket_encryption::BucketEncryption,
    bucket_logging::BucketLogging,
    bucket_versioning::BucketVersioning,
    bucket_website::BucketWebsite,
    public_access_block::PublicAccessBlock,
};
use anyhow::Result;
use rusoto_core::Region;
use rusoto_s3::{
    GetBucketEncryptionRequest,
    GetBucketLoggingRequest,
    GetBucketVersioningRequest,
    GetBucketWebsiteRequest,
    GetPublicAccessBlockRequest,
    S3,
    S3Client,
};

pub struct Client {
    client: S3Client,
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

        let bucket_names = if let Some(buckets) = output.buckets {
            buckets.iter()
                .filter_map(|b| b.name.to_owned())
                .collect()
        }
        else {
            Vec::new()
        };

        Ok(bucket_names)
    }

    async fn get_bucket_encryption(&self, bucket: &str) -> Result<BucketEncryption> {
        let input = GetBucketEncryptionRequest {
            bucket: bucket.to_owned(),
        };

        let output = self.client.get_bucket_encryption(input).await;
        let config: BucketEncryption = output.into();

        Ok(config)
    }

    async fn get_bucket_logging(&self, bucket: &str) -> Result<BucketLogging> {
        let input = GetBucketLoggingRequest {
            bucket: bucket.into(),
        };

        let output = self.client.get_bucket_logging(input).await?;
        let config: BucketLogging = output.into();

        Ok(config)
    }

    async fn get_bucket_versioning(&self, bucket: &str) -> Result<BucketVersioning> {
        let input = GetBucketVersioningRequest {
            bucket: bucket.into(),
        };

        let output = self.client.get_bucket_versioning(input).await?;
        let config: BucketVersioning = output.into();

        Ok(config)
    }

    async fn get_bucket_website(&self, bucket: &str) -> Result<BucketWebsite> {
        let input = GetBucketWebsiteRequest {
            bucket: bucket.into(),
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
        };

        let output = self.client.get_public_access_block(input).await?;
        let config: PublicAccessBlock = output.into();

        Ok(config)
    }

    // Reports on a single bucket
    pub async fn report(&self, bucket: &str) -> Result<()> {
        println!("  {} {}", Emoji::Arrow, bucket);

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

        // Bucket logging
        let bucket_logging = self.get_bucket_logging(bucket).await?;
        println!("    {}", bucket_logging);

        Ok(())
    }

    // Reports on all discovered buckets
    pub async fn report_all(&self) -> Result<()> {
        let buckets = self.list_buckets().await?;

        for bucket in buckets.iter() {
            self.report(&bucket).await?;
        }

        Ok(())
    }
}
