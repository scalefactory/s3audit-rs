// Checks if S3 policies allow wildcard entities.
use anyhow::{anyhow, Result};
use crate::common::Emoji;
use rusoto_s3::GetBucketPolicyOutput;
use serde_json::Value;
use std::fmt;
use std::convert::TryFrom;

mod actions;
mod principals;

use actions::*;
use principals::*;

#[derive(Debug)]
pub struct CloudFrontDistributions(usize);

impl fmt::Display for CloudFrontDistributions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let num = self.0;

        if num == 0 {
            let emoji = Emoji::Tick;

            write!(
                f,
                "{} Bucket is not associated with any CloudFront distributions",
                emoji,
            )
        }
        else {
            let emoji = Emoji::Cross;
            let maybe_plural = if num > 1 {
                "s"
            }
            else {
                ""
            };

            write!(
                f,
                "{} Bucket is associated with {} CloudFront distributions{}",
                emoji,
                num,
                maybe_plural,
            )
        }
    }
}

#[derive(Debug, Default)]
pub struct Wildcards(usize);

impl Wildcards {
    fn add(&mut self, count: usize) {
        self.0 += count;
    }
}

impl fmt::Display for Wildcards {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let num = self.0;

        if num == 0 {
            let emoji = Emoji::Tick;

            write!(
                f,
                "{} Bucket policy doesn't allow a wildcard entity",
                emoji,
            )
        }
        else {
            let emoji = Emoji::Cross;
            let maybe_plural = if num > 1 {
                "s"
            }
            else {
                ""
            };

            write!(
                f,
                "{} Bucket has {} statement{} with wildcard entities",
                emoji,
                num,
                maybe_plural,
            )
        }
    }
}

#[derive(Debug, Default)]
pub struct BucketPolicy {
    actions: Action,
    principals: Principal,
}

impl BucketPolicy {
    pub fn cloudfront_distributions(&self) -> CloudFrontDistributions {
        CloudFrontDistributions(self.principals.cloudfront_distributions())
    }

    pub fn wildcards(&self) -> Wildcards {
        let mut wildcards: Wildcards = Default::default();

        wildcards.add(self.actions.wildcards());
        wildcards.add(self.principals.wildcards());

        wildcards
    }
}

impl TryFrom<GetBucketPolicyOutput> for BucketPolicy {
    type Error = anyhow::Error;

    fn try_from(output: GetBucketPolicyOutput) -> Result<Self, Self::Error> {
        // Caller must have checked that the policy exists; fail otherwise
        let policy = match output.policy {
            None => {
                return Err(anyhow!("Invalid bucket policy"))
            },
            Some(policy_string) => {
                policy_string
            }
        };

        // We expect that AWS will always give us a well formed JSON policy
        let jv: Value = serde_json::from_str(&policy)?;

        // The policy will contain an array of statements.
        let statements = &jv["Statement"];

        let mut actions: Action = Default::default();
        let mut principals: Principal = Default::default();

        let statements_array = statements.as_array()
            .expect("Bucket policy has no Statements element");

        for statement in statements_array.iter() {
            // Policies MUST have an effect. This should never fail.
            let effect = statement["Effect"].as_str()
                .expect("Bucket policy statement does not have an explicit Effect");

            // If we're denying stuff, wildcards are fine and we can proceed
            // to the next statement.
            if effect == "Deny" {
                continue
            }

            // Process the actions.
            let action = &statement["Action"];
            let action: Action = action.into();
            actions.append(action);

            // Process the principals.
            let principal = &statement["Principal"];
            let principal: Principal = principal.into();
            principals.append(principal);
        }

        Ok(Self {
            actions: actions,
            principals: principals,
        })
    }
}

#[derive(Debug, Default)]
pub struct NoBucketPolicy {
}


impl fmt::Display for NoBucketPolicy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} No bucket policy set",
            Emoji::Info,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;
}
