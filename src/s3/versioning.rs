// Bucket versioning
use crate::common::Emoji;
use aws_sdk_s3::model::{
    BucketVersioningStatus,
    MfaDeleteStatus,
};
use aws_sdk_s3::output::GetBucketVersioningOutput;
use std::fmt;

#[derive(Debug, Eq, PartialEq)]
pub enum MfaStatus {
    Enabled,
    Disabled,
}

impl From<MfaDeleteStatus> for MfaStatus {
    fn from(status: MfaDeleteStatus) -> Self {
        match status {
            MfaDeleteStatus::Disabled => Self::Disabled,
            MfaDeleteStatus::Enabled  => Self::Enabled,
            _                         => todo!(),
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

#[derive(Debug, Eq, PartialEq)]
pub enum VersioningStatus {
    Enabled,
    Suspended,
}

impl From<BucketVersioningStatus> for VersioningStatus {
    fn from(status: BucketVersioningStatus) -> Self {
        match status {
            BucketVersioningStatus::Enabled   => Self::Enabled,
            BucketVersioningStatus::Suspended => Self::Suspended,
            _                                 => todo!(),
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

#[derive(Debug, Eq, PartialEq)]
pub struct BucketVersioning {
    mfa_delete: MfaStatus,
    versioning: VersioningStatus,
}

impl From<GetBucketVersioningOutput> for BucketVersioning {
    fn from(output: GetBucketVersioningOutput) -> Self {
        let mfa_delete: MfaStatus = output.mfa_delete
            .map_or(MfaStatus::Disabled, MfaStatus::from);

        let versioning: VersioningStatus = output.status
            .map_or(VersioningStatus::Suspended, VersioningStatus::from);

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
    use aws_sdk_s3::output::GetBucketVersioningOutput;

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

            let output = GetBucketVersioningOutput::builder()
                .mfa_delete(mfa_delete.into())
                .status(status.into())
                .build();

            let expected = BucketVersioning {
                mfa_delete: mfa_delete_expected,
                versioning: versioning_expected,
            };

            let versioning: BucketVersioning = output.into();

            assert_eq!(versioning, expected)
        }
    }
}
