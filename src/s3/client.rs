// S3 client implementation
use crate::common::EMOJI_ARROW;
use crate::s3::public_access_block::PublicAccessBlock;
use anyhow::Result;
use rusoto_core::Region;
use rusoto_s3::{
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
    pub async fn list_buckets(&self) -> Result<Vec<String>> {
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

    // Get the bucket's public access block configuration
    pub async fn get_public_access_block(&self, bucket: &str) -> Result<PublicAccessBlock> {
        let input = GetPublicAccessBlockRequest {
            bucket: bucket.to_owned(),
        };

        let output = self.client.get_public_access_block(input).await?;
        let config: PublicAccessBlock = output.into();

        Ok(config)
    }

    // Reports on a single bucket
    pub async fn report(&self, bucket: &str) -> Result<()> {
        println!("  {} {}", EMOJI_ARROW, bucket);
        println!("    {} Bucket public access configuration", EMOJI_ARROW);

        let public_access_blocks = self.get_public_access_block(bucket).await?;
        for block in public_access_blocks.iter() {
            println!("      {}", block);
        }

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
