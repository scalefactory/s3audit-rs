// Bucket versioning
use crate::common::Emoji;
use rusoto_s3::GetBucketVersioningOutput;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum MfaStatus {
    Enabled,
    Disabled,
}

impl From<String> for MfaStatus {
    fn from(status: String) -> Self {
        match status.as_ref() {
            "Enabled"  => Self::Enabled,
            "Disabled" => Self::Disabled,
            _          => unreachable!(),
        }
    }
}

impl fmt::Display for MfaStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = match self {
            Self::Enabled => {
                let emoji = Emoji::Tick;
                format!("{} MFA Delete is enabled", emoji)
            },
            Self::Disabled => {
                let emoji = Emoji::Cross;
                format!("{} MFA Delete is not enabled", emoji)
            },
        };

        write!(f, "{}", output)
    }
}

#[derive(Debug, PartialEq)]
pub enum VersioningStatus {
    Enabled,
    Suspended,
}

impl From<String> for VersioningStatus {
    fn from(status: String) -> Self {
        match status.as_ref() {
            "Enabled"   => Self::Enabled,
            "Suspended" => Self::Suspended,
            _           => unreachable!(),
        }
    }
}

impl fmt::Display for VersioningStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = match self {
            Self::Enabled => {
                let emoji = Emoji::Tick;
                format!("{} Object Versioning is enabled", emoji)
            },
            Self::Suspended => {
                let emoji = Emoji::Cross;
                format!("{} Object Versioning is not enabled", emoji)
            },
        };

        write!(f, "{}", output)
    }
}

#[derive(Debug, PartialEq)]
pub struct BucketVersioning {
    mfa_delete: MfaStatus,
    versioning: VersioningStatus,
}

impl From<GetBucketVersioningOutput> for BucketVersioning {
    fn from(output: GetBucketVersioningOutput) -> Self {
        let mfa_delete: MfaStatus = output.mfa_delete
            .map_or(MfaStatus::Disabled, |mfa_delete| mfa_delete.into());

        let versioning: VersioningStatus = output.status
            .map_or(VersioningStatus::Suspended, |status| status.into());

        Self {
            mfa_delete: mfa_delete,
            versioning: versioning,
        }
    }
}

impl BucketVersioning {
    pub fn mfa_delete(&self) -> &MfaStatus {
        &self.mfa_delete
    }

    pub fn versioning(&self) -> &VersioningStatus {
        &self.versioning
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_for_bucket_versioning() {
        let tests = vec![
            ("Enabled", "Enabled", MfaStatus::Enabled, VersioningStatus::Enabled),
            ("Disabled", "Suspended", MfaStatus::Disabled, VersioningStatus::Suspended),
        ];

        for test in tests {
            let mfa_delete          = test.0;
            let status              = test.1;
            let mfa_delete_expected = test.2;
            let versioning_expected = test.3;

            let output = GetBucketVersioningOutput {
                mfa_delete: Some(mfa_delete.into()),
                status:     Some(status.into()),
            };

            let expected = BucketVersioning {
                mfa_delete: mfa_delete_expected,
                versioning: versioning_expected,
            };

            let versioning: BucketVersioning = output.into();

            assert_eq!(versioning, expected)
        }
    }
}
