// S3 client implementation
use crate::s3::{
    acl::BucketAcl,
    audits::Audit,
    encryption::BucketEncryption,
    logging::BucketLogging,
    policy::BucketPolicy,
    public_access_block::PublicAccessBlock,
    versioning::BucketVersioning,
    website::BucketWebsite,
    Report,
    Reports,
};
use anyhow::Result;
use log::info;
use rusoto_core::{
    Region,
    RusotoError,
};
use rusoto_s3::{
    GetBucketAclRequest,
    GetBucketEncryptionRequest,
    GetBucketLocationRequest,
    GetBucketLoggingRequest,
    GetBucketPolicyRequest,
    GetBucketVersioningRequest,
    GetBucketWebsiteRequest,
    GetPublicAccessBlockRequest,
    S3,
    S3Client,
};
use std::convert::TryInto;
use std::fmt;
use std::str::FromStr;

#[derive(Debug)]
struct Bucket {
    name:   String,
    region: Region,
}

impl fmt::Display for Bucket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub struct Client {
    client: S3Client,
}

impl Client {
    // Get a new S3 client
    pub fn new(region: Option<Region>) -> Self {
        let region = region.map_or_else(Region::default, |region| region);

        info!("Creating new client in region: {:?}", region);

        let client = S3Client::new(region);

        Self {
            client: client,
        }
    }

    // List all buckets on an account
    // We also get the region for each bucket here as we need to create the S3
    // client for each bucket in the appropriate location.
    async fn list_buckets(&self) -> Result<Vec<Bucket>> {
        info!("Listing buckets");

        let output = self.client.list_buckets().await?;

        let bucket_names = output.buckets.map_or_else(Vec::new, |buckets| {
            buckets.iter()
                .filter_map(|bucket| bucket.name.to_owned())
                .collect()
        });

        let mut buckets: Vec<Bucket> = Vec::new();

        for bucket in bucket_names {
            let region = self.get_bucket_region(&bucket).await?;
            let bucket = Bucket {
                name:   bucket,
                region: region,
            };

            buckets.push(bucket);
        }

        Ok(buckets)
    }

    async fn get_bucket_acl(&self, bucket: &str) -> Result<BucketAcl> {
        info!("Getting bucket ACL for bucket: {}", bucket);

        let input = GetBucketAclRequest {
            bucket: bucket.into(),
            ..Default::default()
        };

        let output = self.client.get_bucket_acl(input).await?;
        let config: BucketAcl = output.into();

        Ok(config)
    }

    async fn get_bucket_encryption(&self, bucket: &str) -> Result<BucketEncryption> {
        info!("Getting bucket encryption for bucket: {}", bucket);

        let input = GetBucketEncryptionRequest {
            bucket: bucket.to_owned(),
            ..Default::default()
        };

        let output = self.client.get_bucket_encryption(input).await;
        let config: BucketEncryption = output.into();

        Ok(config)
    }

    async fn get_bucket_location(&self, bucket: &str) -> Result<String> {
        info!("Getting bucket location for bucket: {}", bucket);

        let input = GetBucketLocationRequest {
            bucket: bucket.into(),
            ..Default::default()
        };

        let output = self.client.get_bucket_location(input).await?;
        let location = output.location_constraint
            .unwrap_or_else(|| "us-east-1".into());

        // us-east-1 is special and returns null as its location.
        // eu-west-1 may return EU as its location depending on bucket age.
        let location = match location.as_ref() {
            ""   => "us-east-1".to_string(),
            "EU" => "eu-west-1".to_string(),
            _    => location,
        };

        Ok(location)
    }

    async fn get_bucket_logging(&self, bucket: &str) -> Result<BucketLogging> {
        info!("Getting bucket logging for bucket: {}", bucket);

        let input = GetBucketLoggingRequest {
            bucket: bucket.into(),
            ..Default::default()
        };

        let output = self.client.get_bucket_logging(input).await?;
        let config: BucketLogging = output.into();

        Ok(config)
    }

    async fn get_bucket_policy(&self, bucket: &str) -> Result<Option<BucketPolicy>> {
        info!("Getting bucket policy for bucket: {}", bucket);

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

    async fn get_bucket_region(&self, bucket: &str) -> Result<Region> {
        info!("Getting bucket region for bucket: {}", bucket);

        let location = self.get_bucket_location(bucket).await?;
        info!("  - Bucket location is: {}", location);

        let region   = Region::from_str(&location)?;

        Ok(region)
    }

    async fn get_bucket_versioning(&self, bucket: &str) -> Result<BucketVersioning> {
        info!("Getting bucket versioning for bucket: {}", bucket);

        let input = GetBucketVersioningRequest {
            bucket: bucket.into(),
            ..Default::default()
        };

        let output = self.client.get_bucket_versioning(input).await?;
        let config: BucketVersioning = output.into();

        Ok(config)
    }

    async fn get_bucket_website(&self, bucket: &str) -> Result<BucketWebsite> {
        info!("Getting bucket website for bucket: {}", bucket);

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
        info!("Getting public access block for bucket: {}", bucket);

        let input = GetPublicAccessBlockRequest {
            bucket: bucket.to_owned(),
            ..Default::default()
        };

        let output = self.client.get_public_access_block(input).await;

        // The error here should be more closely inspected. For now we just
        // assume that the PublicAccessBlock settings didn't exist, so they
        // aren't set.
        let config = match output {
            Err(_) => PublicAccessBlock::default(),
            Ok(o)  => o.into(),
        };

        Ok(config)
    }

    // Reports on a single bucket
    async fn bucket_report(
        &self,
        bucket: &str,
        audits: &[Audit],
    ) -> Result<Report> {
        info!("Generating report for bucket: {}", bucket);

        let acl = if audits.contains(&Audit::Acl) {
            let resp = self.get_bucket_acl(bucket).await?;
            Some(resp)
        }
        else {
            None
        };

        let encryption = if audits.contains(&Audit::ServerSideEncryption) {
            let resp = self.get_bucket_encryption(bucket).await?;
            Some(resp)
        }
        else {
            None
        };

        let logging = if audits.contains(&Audit::Logging) {
            let resp = self.get_bucket_logging(bucket).await?;
            Some(resp)
        }
        else {
            None
        };

        let policy = if audits.contains(&Audit::Policy) {
            let resp = self.get_bucket_policy(bucket).await?;
            Some(resp)
        }
        else {
            None
        };

        let public_access_block = if audits.contains(&Audit::PublicAccessBlocks) {
            let resp = self.get_public_access_block(bucket).await?;
            Some(resp)
        }
        else {
            None
        };

        // Both of these come from the Versioning API, so enabled either of
        // these needs to get the bucket versioning.
        let versioning_audits = vec![
            Audit::MfaDelete,
            Audit::Versioning,
        ];

        let audit_versioning = versioning_audits
            .iter()
            .any(|x| audits.contains(x));

        let versioning = if audit_versioning {
            let resp = self.get_bucket_versioning(bucket).await?;
            Some(resp)
        }
        else {
            None
        };

        let website = if audits.contains(&Audit::Website) {
            let resp = self.get_bucket_website(bucket).await?;
            Some(resp)
        }
        else {
            None
        };

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
    pub async fn report(
        &self,
        bucket: Option<String>,
        audits: Vec<Audit>,
    ) -> Result<Reports> {
        let buckets = match bucket {
            None         => self.list_buckets().await?,
            Some(bucket) => {
                let region = self.get_bucket_region(&bucket).await?;
                let bucket = Bucket {
                    name: bucket,
                    region: region,
                };

                vec![bucket]
            },
        };

        info!("Generating reports for buckets: {:?}", buckets);

        let mut reports = Vec::new();

        // We get a new client for each bucket, as we must interact with
        // buckets from the region they reside in.
        for bucket in buckets.iter() {
            let region = Some(bucket.region.to_owned());
            let client = Self::new(region);
            let report = client.bucket_report(&bucket.name, &audits).await?;

            reports.push(report);
        }

        let reports = Reports::new(reports);

        Ok(reports)
    }
}
