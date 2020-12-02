// Checks if S3 policies allow wildcard entities.
use crate::common::Emoji;
use rusoto_s3::GetBucketPolicyOutput;
use serde_json::Value;
use std::fmt;

const WILDCARD: &str = "*";

#[derive(Debug, PartialEq)]
pub enum BucketPolicy {
    None,
    NoWildcards,
    Wildcards(usize),
}

#[derive(Debug)]
struct Action(Vec<String>);

impl Action {
    fn wildcards(&self) -> usize {
        // Wildcards could appear anywhere in the ARN
        // eg. "*", "s3:*", "iam:*AccessKey*"
        self.0.iter()
            .filter(|&arn| arn.contains(WILDCARD))
            .count()
    }
}

#[cfg(test)]
impl PartialEq for Action {
    fn eq(&self, other: &Self) -> bool {
        let mut first = self.0.clone();
        first.sort();

        let mut second = other.0.clone();
        second.sort();

        first == second
    }
}

// Takes a Value representing the Principal entry in a Bucket Policy and
// returns a Vec of the discovered ARNs wrapped in a Principal struct.
impl From<&Value> for Action {
    fn from(value: &Value) -> Self {
        let output = match value {
            // "Action": "s3:Foo"
            Value::String(arn) => {
                let arns = vec![
                    String::from(arn),
                ];

                Self(arns)
            },
            // "Action": [
            //   "s3:Bar",
            //   "s3:Foo",
            // ]
            Value::Array(actions) => {
                let actions: Vec<String> = actions.iter()
                    .map(|s| String::from(s.as_str().unwrap()))
                    .collect();

                Self(actions)
            },
            _ => Self(Vec::new()),
        };

        output
    }
}

#[derive(Debug)]
struct Principal(Vec<String>);

impl Principal {
    fn wildcards(&self) -> usize {
        self.0.iter()
            .filter(|&arn| arn == WILDCARD)
            .count()
    }
}

// Takes a Value representing the Principal entry in a Bucket Policy and
// returns a Vec of the discovered ARNs wrapped in a Principal struct.
impl From<&Value> for Principal {
    fn from(value: &Value) -> Self {
        let output = match value {
            // "Principal": "arn:aws:iam::etc"
            Value::String(arn) => {
                let arns = vec![
                    String::from(arn),
                ];

                Self(arns)
            },
            // "Principal": {
            //   "AWS": [
            //     "arn:aws:iam::foo",
            //     "123456789012",
            //     "*"
            //   ]
            // }
            // or
            // "Principal": {
            //   "AWS": "arn:aws:iam::foo"
            // }
            Value::Object(o) => {
                // This could also be "Federated", "Service", "CanonicalUser",
                // etc, but we aren't interested in those.
                match &o["AWS"] {
                    Value::String(arn) => {
                        let arns = vec![
                            String::from(arn),
                        ];

                        Self(arns)
                    },
                    Value::Array(vec) => {
                        // Each entry should be a string now.
                        let arns: Vec<String> = vec.iter()
                            .map(|s| String::from(s.as_str().unwrap()))
                            .collect();

                        Self(arns)
                    },
                    _ => Self(Vec::new()),
                }
            },
            _ => Self(Vec::new()),
        };

        output
    }
}

impl From<GetBucketPolicyOutput> for BucketPolicy {
    fn from(output: GetBucketPolicyOutput) -> Self {
        if output.policy.is_none() {
            return Self::None;
        }

        // We already checked if the policy exists, unwrap should be fine
        let policy = output.policy.unwrap();

        // We expect that AWS will always give us a wellformed JSON policy
        let jv: Value = serde_json::from_str(&policy)
            .expect("Invalid JSON from AWS");

        // Policies MUST have an effect. This should be safe.
        let effect = String::from(jv["Effect"].as_str().unwrap());

        // If we're denying stuff, wildcards are fine.
        if effect.to_lowercase() == "deny" {
            return Self::NoWildcards;
        }

        let mut wildcard_statements_total: usize = 0;

        // Process the actions.
        let action = &jv["Action"];
        let action: Action = action.into();
        let action_wildcards = action.wildcards();

        wildcard_statements_total += action_wildcards;

        // Process the principals.
        let principal = &jv["Principal"];
        let principal: Principal = principal.into();
        let principal_wildcards = principal.wildcards();

        wildcard_statements_total += principal_wildcards;

        if wildcard_statements_total == 0 {
            return Self::NoWildcards;
        }

        Self::Wildcards(wildcard_statements_total)
    }
}

impl fmt::Display for BucketPolicy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = match *self {
            Self::None => {
                let emoji = Emoji::Warning;

                format!("{} Bucket has no policy set", emoji)
            },
            Self::NoWildcards => {
                let emoji = Emoji::Tick;

                format!(
                    "{} Bucket policy doesn't allow a wildcard entity",
                    emoji,
                )
            },
            Self::Wildcards(num) => {
                let emoji = Emoji::Cross;
                let maybe_plural = if num as usize > 1 {
                    "s"
                }
                else {
                    ""
                };

                format!(
                    "{} Bucket has {} statement{} with wildcard entities",
                    emoji,
                    num,
                    maybe_plural,
                )
            },
        };

        write!(f, "{}", output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn test_from_policy_action_string() {
        let policy = json!({
            "Effect": "Allow",
            "Action": "*",
            "Principal": "*",
        });

        let action = &policy["Action"];
        let action: Action = action.into();
        let expected = vec!["*"];

        assert_eq!(action.0, expected);
        assert_eq!(action.wildcards(), 1);
    }

    #[test]
    fn test_from_policy_action_vec() {
        let policy = json!({
            "Effect": "Allow",
            "Action": [
                "s3:ListAllMyBuckets",
                "s3:GetObject",
                "s3:*",
                "*",
            ],
            "Principal": {
                "AWS": [
                    "arn:aws:iam::123456789012:root",
                    "123456789012",
                    "*",
                ],
            },
        });

        let action = &policy["Action"];
        let action: Action = action.into();
        let expected = Action(vec![
            "s3:GetObject".into(),
            "s3:ListAllMyBuckets".into(),
            "s3:*".into(),
            "*".into(),
        ]);

        assert_eq!(action, expected);
        assert_eq!(action.wildcards(), 2);
    }

    #[test]
    fn test_from_policy_principal_string() {
        let policy = json!({
            "Effect": "Allow",
            "Action": "*",
            "Principal": "*",
        });

        let principal = &policy["Principal"];
        let principal: Principal = principal.into();
        let expected = vec!["*"];

        assert_eq!(principal.0, expected);
    }

    #[test]
    fn test_from_policy_principal_vec() {
        let policy = json!({
            "Effect": "Allow",
            "Action": "*",
            "Principal": {
                "AWS": [
                    "arn:aws:iam::123456789012:root",
                    "123456789012",
                    "*",
                ],
            },
        });

        let principal = &policy["Principal"];
        let principal: Principal = principal.into();
        let expected = vec![
            "arn:aws:iam::123456789012:root",
            "123456789012",
            "*",
        ];

        assert_eq!(principal.0, expected);
    }
}
