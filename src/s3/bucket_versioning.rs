// Bucket versioning
use crate::common::Emoji;
use rusoto_s3::GetBucketVersioningOutput;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum MFAStatus {
    Enabled,
    Disabled,
}

impl From<String> for MFAStatus {
    fn from(status: String) -> Self {
        match status.as_ref() {
            "Enabled"  => Self::Enabled,
            "Disabled" => Self::Disabled,
            _          => unreachable!(),
        }
    }
}

impl fmt::Display for MFAStatus {
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
    mfa_delete: MFAStatus,
    versioning: VersioningStatus,
}

impl From<GetBucketVersioningOutput> for BucketVersioning {
    fn from(output: GetBucketVersioningOutput) -> Self {
        let mfa_delete: MFAStatus = if let Some(mfa) = output.mfa_delete {
            mfa.into()
        }
        else {
            MFAStatus::Disabled
        };

        let versioning: VersioningStatus = if let Some(status) = output.status {
            status.into()
        }
        else {
            VersioningStatus::Suspended
        };

        Self {
            mfa_delete: mfa_delete,
            versioning: versioning,
        }
    }
}

impl BucketVersioning {
    pub fn mfa_delete(&self) -> &MFAStatus {
        &self.mfa_delete
    }

    pub fn versioning(&self) -> &VersioningStatus {
        &self.versioning
    }
}
