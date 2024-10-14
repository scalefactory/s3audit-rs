// Bucket ACL
use crate::common::Emoji;
use aws_sdk_s3::operation::get_bucket_acl::GetBucketAclOutput;
use std::fmt;

// Grantee URIs that indicate public access
const PUBLIC_URIS: &[&str] = &[
    "http://acs.amazonaws.com/groups/global/AllUsers",
    "http://acs.amazonaws.com/groups/global/AuthenticatedUsers",
];

#[derive(Debug, Eq, PartialEq)]
pub enum BucketAcl {
    Private,
    Public,
}

impl From<GetBucketAclOutput> for BucketAcl {
    fn from(output: GetBucketAclOutput) -> Self {
        let grants = output.grants();

        // Might have no grants
        if grants.is_empty() {
            return Self::Private;
        }

        // Loop over grants checking for public URIs
        for grant in grants {
            let Some(grantee) = grant.grantee() else {
                continue;
            };

            let Some(uri) = grantee.uri() else {
                continue;
            };

            if PUBLIC_URIS.contains(&uri) {
                return Self::Public;
            }
        }

        Self::Private
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

#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_s3::types::{
        Grant,
        Grantee,
        Owner,
        Permission,
        Type,
    };

    const PRIVATE_GROUP: &str = "http://acs.amazonaws.com/groups/private/lovelace";
    const PUBLIC_GROUP: &str = PUBLIC_URIS[0];

    #[test]
    fn test_from_for_bucket_acl_private() {
        let owner = Owner::builder()
            .display_name("Ada Lovelace")
            .id("lovelace")
            .build();

        let grantee = Grantee::builder()
            .display_name("Ada Lovelace")
            .email_address("lovelace@example.org")
            .id("lovelace")
            .r#type(Type::from("N/A"))
            .uri(PRIVATE_GROUP)
            .build()
            .unwrap();

        let grant = Grant::builder()
            .grantee(grantee)
            .permission(Permission::from("private"))
            .build();

        let output = GetBucketAclOutput::builder()
            .grants(grant)
            .owner(owner)
            .build();

        let expected = BucketAcl::Private;

        let bucket_acl: BucketAcl = output.into();

        assert_eq!(bucket_acl, expected)
    }

    #[test]
    fn test_from_for_bucket_acl_public() {
        let owner = Owner::builder()
            .display_name("Ada Lovelace")
            .id("lovelace")
            .build();

        let grantee = Grantee::builder()
            .display_name("Ada Lovelace")
            .email_address("lovelace@example.org")
            .id("lovelace")
            .r#type(Type::from("N/A"))
            .uri(PUBLIC_GROUP)
            .build()
            .unwrap();

        let grant = Grant::builder()
            .grantee(grantee)
            .permission(Permission::from("public"))
            .build();

        let output = GetBucketAclOutput::builder()
            .set_grants(Some(vec![grant]))
            .owner(owner)
            .build();

        let expected = BucketAcl::Public;

        let bucket_acl: BucketAcl = output.into();

        assert_eq!(bucket_acl, expected)
    }
}
