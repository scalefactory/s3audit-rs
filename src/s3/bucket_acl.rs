// Bucket ACL
use crate::common::Emoji;
use rusoto_s3::GetBucketAclOutput;
use std::fmt;

// Grantee URIs that indicate public access
const PUBLIC_URIS: &[&str] = &[
    "http://acs.amazonaws.com/groups/global/AllUsers",
    "http://acs.amazonaws.com/groups/global/AuthenticatedUsers",
];

#[derive(Debug, PartialEq)]
pub enum BucketAcl {
    Private,
    Public,
}

impl From<GetBucketAclOutput> for BucketAcl {
    fn from(output: GetBucketAclOutput) -> Self {
        if let Some(grants) = output.grants {
            // Might have no grants
            if grants.is_empty() {
                return Self::Private;
            }

            // Loop over grants checking for public URIs
            for grant in grants {
                if let Some(grantee) = grant.grantee {
                    match grantee.uri {
                        None      => {},
                        Some(uri) => {
                            if PUBLIC_URIS.contains(&&*uri) {
                                return Self::Public;
                            }
                        },
                    }
                }
            }

            Self::Private
        }
        else {
            Self::Private
        }
    }
}

impl fmt::Display for BucketAcl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = match self {
            Self::Private => {
                let emoji = Emoji::Tick;
                format!("{} Bucket ACL doesn't allow access to 'Everyone' or \
                         'Any authenticated AWS user'", emoji)
            },
            Self::Public => {
                let emoji = Emoji::Warning;
                format!("{} Bucket allows public access via ACL", emoji)
            }
        };

        write!(f, "{}", output)
    }
}
