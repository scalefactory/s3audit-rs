// Implements a nice enum for expressing public access block status
use crate::common::Emoji;
use aws_sdk_s3::operation::get_public_access_block::GetPublicAccessBlockOutput;
use std::fmt;
use std::ops::Deref;

#[derive(Debug, Eq, PartialEq)]
pub enum PublicAccessBlockType {
    BlockPublicAcls(bool),
    BlockPublicPolicy(bool),
    IgnorePublicAcls(bool),
    RestrictPublicBuckets(bool),
}

impl fmt::Display for PublicAccessBlockType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = match *self {
            Self::BlockPublicAcls(b) => {
                let emoji: Emoji = b.into();
                format!("{} BlockPublicAcls is set to {}", emoji, b)
            },
            Self::BlockPublicPolicy(b) => {
                let emoji: Emoji = b.into();
                format!("{} BlockPublicPolicy is set to {}", emoji, b)
            },
            Self::IgnorePublicAcls(b) => {
                let emoji: Emoji = b.into();
                format!("{} IgnorePublicAcls is set to {}", emoji, b)
            },
            Self::RestrictPublicBuckets(b) => {
                let emoji: Emoji = b.into();
                format!("{} RestrictPublicBuckets is set to {}", emoji, b)
            },
        };

        write!(f, "{}", output)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct PublicAccessBlock(Vec<PublicAccessBlockType>);

impl Default for PublicAccessBlock {
    fn default() -> Self {
        let blocks = vec![
            PublicAccessBlockType::BlockPublicAcls(false),
            PublicAccessBlockType::BlockPublicPolicy(false),
            PublicAccessBlockType::IgnorePublicAcls(false),
            PublicAccessBlockType::RestrictPublicBuckets(false),
        ];

        Self(blocks)
    }
}

impl From<GetPublicAccessBlockOutput> for PublicAccessBlock {
    fn from(output: GetPublicAccessBlockOutput) -> Self {
        let config = output.public_access_block_configuration
            .expect("public_access_block_configuration");

        let block_public_acls = config.block_public_acls.unwrap_or(false);
        let block_public_policy = config.block_public_policy.unwrap_or(false);
        let ignore_public_acls = config.ignore_public_acls.unwrap_or(false);
        let restrict_public_buckets = config.restrict_public_buckets.unwrap_or(false);

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
    use aws_sdk_s3::types::PublicAccessBlockConfiguration;

    #[test]
    fn test_from() {
        let public_access_block_configuration = PublicAccessBlockConfiguration::builder()
            .block_public_acls(true)
            .block_public_policy(false)
            .ignore_public_acls(true)
            .restrict_public_buckets(false)
            .build();

        let output = GetPublicAccessBlockOutput::builder()
            .public_access_block_configuration(public_access_block_configuration)
            .build();

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
