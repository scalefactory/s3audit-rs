// S3 client implementation
use crate::s3::{
    acl::BucketAcl,
    encryption::BucketEncryption,
    logging::BucketLogging,
    policy::BucketPolicy,
    public_access_block::PublicAccessBlock,
    versioning::BucketVersioning,
    website::BucketWebsite,
    Report,
};
use anyhow::Result;
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

        let bucket_policy: BucketPolicy = output.try_into()?;
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
    pub async fn report(&self, bucket: &str) -> Result<Report> {
        let acl = self.get_bucket_acl(bucket).await?;
        let encryption = self.get_bucket_encryption(bucket).await?;
        let logging = self.get_bucket_logging(bucket).await?;
        let policy = self.get_bucket_policy(bucket).await?;
        let public_access_block = self.get_public_access_block(bucket).await?;
        let versioning = self.get_bucket_versioning(bucket).await?;
        let website = self.get_bucket_website(bucket).await?;

        let report = Report {
            name:                bucket.into(),
            acl:                 acl,
            encryption:          encryption,
            logging:             logging,
            policy:              policy,
            public_access_block: public_access_block,
            versioning:          versioning,
            website:             website,
        };

        Ok(report)
    }

    // Reports on all discovered buckets
    pub async fn report_all(&self) -> Result<Vec<Report>> {
        let buckets = self.list_buckets().await?;
        let mut reports = Vec::new();

        for bucket in buckets.iter() {
            let report = self.report(&bucket).await?;
            reports.push(report);
        }

        Ok(reports)
    }
}
