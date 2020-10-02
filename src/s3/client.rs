// S3 client implementation
use crate::common::Emoji;
use crate::s3::{
    bucket_encryption::BucketEncryption,
    public_access_block::PublicAccessBlock,
};
use anyhow::Result;
use rusoto_core::Region;
use rusoto_s3::{
    GetBucketEncryptionRequest,
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

        // We might get an Err here for two reasons:
        //   - Encryption isn't enabled
        //   - We don't have the s3:GetEncryptionConfiguration permission
        //
        // In the future it would be good to distinguish between the two, but
        // that will involve some parsing of the XML message returned.
        //
        // For now, we'll just assert that there is no encryption and the user
        // can verify it for themselves.
        let output = match self.client.get_bucket_encryption(input).await {
            Ok(res) => BucketEncryption::from(res),
            Err(_)  => BucketEncryption::None,
        };

        Ok(output)
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
