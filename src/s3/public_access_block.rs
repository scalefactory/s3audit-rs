// Implements a nice enum for expressing public access block status
use crate::common::{
    EMOJI_CROSS,
    EMOJI_TICK,
};
use rusoto_s3::GetPublicAccessBlockOutput;
use std::fmt;
use std::ops::Deref;

#[derive(Debug)]
pub enum PublicAccessBlockType {
    BlockPublicAcls(bool),
    BlockPublicPolicy(bool),
    IgnorePublicAcls(bool),
    RestrictPublicBuckets(bool),
}

// Get the appropriate emoji for true or false status
fn emoji<'a>(b: &bool) -> &'a str {
    if *b {
        EMOJI_TICK
    }
    else {
        EMOJI_CROSS
    }
}

impl fmt::Display for PublicAccessBlockType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = match self {
            Self::BlockPublicAcls(b) => {
                format!("{} BlockPublicAcls is set to {}", emoji(b), b)
            },
            Self::BlockPublicPolicy(b) => {
                format!("{} BlockPublicPolicy is set to {}", emoji(b), b)
            },
            Self::IgnorePublicAcls(b) => {
                format!("{} IgnorePublicAcls is set to {}", emoji(b), b)
            },
            Self::RestrictPublicBuckets(b) => {
                format!("{} RestrictPublicBuckets is set to {}", emoji(b), b)
            },
        };

        write!(f, "{}", output)
    }
}

#[derive(Debug)]
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
