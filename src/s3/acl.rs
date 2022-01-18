// Bucket ACL
use crate::common::Emoji;
use aws_sdk_s3::output::GetBucketAclOutput;
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
        let grants = match output.grants {
            None         => return Self::Private,
            Some(grants) => grants,
        };

        // Might have no grants
        if grants.is_empty() {
            return Self::Private;
        }

        // Loop over grants checking for public URIs
        for grant in grants {
            let grantee = match grant.grantee {
                None          => continue,
                Some(grantee) => grantee,
            };

            let uri = match grantee.uri {
                None      => continue,
                Some(uri) => uri,
            };

            if PUBLIC_URIS.contains(&&*uri) {
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
    use aws_sdk_s3::model::{
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
            .r#type(Type::Unknown("N/A".into()))
            .uri(PRIVATE_GROUP)
            .build();

        let grant = Grant::builder()
            .grantee(grantee)
            .permission(Permission::Unknown("private".into()))
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
            .r#type(Type::Unknown("N/A".into()))
            .uri(PUBLIC_GROUP)
            .build();

        let grant = Grant::builder()
            .grantee(grantee)
            .permission(Permission::Unknown("public".into()))
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
