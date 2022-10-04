// Audit types
use anyhow::{
    anyhow,
    Error,
};
use std::collections::HashSet;
use std::str::FromStr;

// Quickly create a HashSet, in the style of a vec![]
macro_rules! hashset {
    ( $( $entry:expr ),* $(,)? ) => {
        {
            let mut macro_set = HashSet::new();

            $(
                macro_set.insert($entry);
            )*

            macro_set
        }
    };
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Audit {
    Acl,
    All,
    Cloudfront,
    Logging,
    MfaDelete,
    Policy,
    PublicAccessBlocks,
    ServerSideEncryption,
    Versioning,
    Website,
}

impl FromStr for Audit {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();

        match s.as_str() {
            // Names matching the above options
            "acl"                   => Ok(Self::Acl),
            "all"                   => Ok(Self::All),
            "cloudfront"            => Ok(Self::Cloudfront),
            "logging"               => Ok(Self::Logging),
            "policy"                => Ok(Self::Policy),
            "public-access-blocks"  => Ok(Self::PublicAccessBlocks),
            "versioning"            => Ok(Self::Versioning),
            "website"               => Ok(Self::Website),

            // Aliases
            "encryption" | "server-side-encryption" | "sse" => {
                Ok(Self::ServerSideEncryption)
            },
            "mfa" | "mfa-delete" => {
                Ok(Self::MfaDelete)
            },

            // Invalid audits
            _ => Err(anyhow!("Unknown Report Type")),
        }
    }
}

impl Default for Audit {
    fn default() -> Self {
        Self::All
    }
}

#[derive(Clone, Debug)]
pub struct Audits(HashSet<Audit>);

impl Default for Audits {
    fn default() -> Self {
        // All audits are enabled by default
        let set = hashset![
            Audit::Acl,
            Audit::Cloudfront,
            Audit::Logging,
            Audit::MfaDelete,
            Audit::Policy,
            Audit::PublicAccessBlocks,
            Audit::ServerSideEncryption,
            Audit::Versioning,
            Audit::Website,
        ];

        Self(set)
    }
}

impl Audits {
    // By default all audits are enabled
    pub fn new() -> Self {
        Self::default()
    }

    // Returns an empty set
    fn empty(self) -> Self {
        Self(HashSet::new())
    }

    // Removes audits from the set, disabling them
    pub fn disable(mut self, audits: Option<Vec<Audit>>) -> Self {
        if let Some(audits) = audits {
            // If all audits were disabled, short circuit and just return a new
            // empty Audits struct.
            if audits.contains(&Audit::All) {
                return self.empty();
            }

            for audit in audits {
                self.0.remove(&audit);
            }
        }

        self
    }

    // Adds audits to the set, enabling them
    pub fn enable(mut self, audits: Option<Vec<Audit>>) -> Self {
        if let Some(audits) = audits {
            // If all audits were enabled, short circuit and just return a new
            // full Audits struct.
            if audits.contains(&Audit::All) {
                return Self::new();
            }

            for audit in audits {
                self.0.insert(audit);
            }
        }

        self
    }

    // Returns a vec of enabled audits, those that are left in the set.
    // Audits is consumed.
    pub fn enabled(self) -> Vec<Audit> {
        let mut audits = Vec::new();

        for audit in self.0 {
            audits.push(audit);
        }

        audits
    }
}
