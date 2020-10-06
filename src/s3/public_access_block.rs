// Implements a nice enum for expressing public access block status
use crate::common::*;
use rusoto_s3::GetPublicAccessBlockOutput;
use std::fmt;
use std::ops::Deref;

#[derive(Debug, PartialEq)]
pub enum PublicAccessBlockType {
    BlockPublicAcls(bool),
    BlockPublicPolicy(bool),
    IgnorePublicAcls(bool),
    RestrictPublicBuckets(bool),
}

impl fmt::Display for PublicAccessBlockType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = match self {
            Self::BlockPublicAcls(b) => {
                let emoji = Into::<EmojiBool>::into(*b);
                format!("{} BlockPublicAcls is set to {}", emoji, b)
            },
            Self::BlockPublicPolicy(b) => {
                let emoji = Into::<EmojiBool>::into(*b);
                format!("{} BlockPublicPolicy is set to {}", emoji, b)
            },
            Self::IgnorePublicAcls(b) => {
                let emoji = Into::<EmojiBool>::into(*b);
                format!("{} IgnorePublicAcls is set to {}", emoji, b)
            },
            Self::RestrictPublicBuckets(b) => {
                let emoji = Into::<EmojiBool>::into(*b);
                format!("{} RestrictPublicBuckets is set to {}", emoji, b)
            },
        };

        write!(f, "{}", output)
    }
}

#[derive(Debug, PartialEq)]
pub struct PublicAccessBlock(Vec<PublicAccessBlockType>);

impl From<GetPublicAccessBlockOutput> for PublicAccessBlock {
    fn from(output: GetPublicAccessBlockOutput) -> Self {
        let config = output.public_access_block_configuration
            .expect("public_access_block_configuration");

        let block_public_acls = config.block_public_acls.unwrap();
        let block_public_policy = config.block_public_policy.unwrap();
        let ignore_public_acls = config.ignore_public_acls.unwrap();
        let restrict_public_buckets = config.restrict_public_buckets.unwrap();

        let blocks = vec![
            PublicAccessBlockType::BlockPublicAcls(block_public_acls),
            PublicAccessBlockType::BlockPublicPolicy(block_public_policy),
            PublicAccessBlockType::IgnorePublicAcls(ignore_public_acls),
            PublicAccessBlockType::RestrictPublicBuckets(restrict_public_buckets),
        ];

        PublicAccessBlock(blocks)
    }
}

// Allows us to directly iterate over the struct inner.
impl Deref for PublicAccessBlock {
    type Target = Vec<PublicAccessBlockType>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusoto_s3::PublicAccessBlockConfiguration;

    #[test]
    fn test_from() {
        let public_access_block_configuration = PublicAccessBlockConfiguration {
            block_public_acls: Some(true),
            block_public_policy: Some(false),
            ignore_public_acls: Some(true),
            restrict_public_buckets: Some(false),
        };

        let output = GetPublicAccessBlockOutput {
            public_access_block_configuration: Some(public_access_block_configuration),
        };

        let expected = PublicAccessBlock(vec![
            PublicAccessBlockType::BlockPublicAcls(true),
            PublicAccessBlockType::BlockPublicPolicy(false),
            PublicAccessBlockType::IgnorePublicAcls(true),
            PublicAccessBlockType::RestrictPublicBuckets(false),
        ]);

        let public_access_block: PublicAccessBlock = output.into();

        assert_eq!(public_access_block, expected)
    }
}
