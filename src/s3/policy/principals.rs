// Checks if S3 policies allow wildcard entities.
use log::debug;
use serde_json::Value;

const CLOUDFRONT_OAI: &str = "arn:aws:iam::cloudfront:user/CloudFront Origin Access Identity ";
const WILDCARD: &str = "*";

#[derive(Debug, Default)]
pub struct Principal(Vec<String>);

impl Principal {
    pub fn append(&mut self, mut other: Self) {
        self.0.append(&mut other.0);
    }

    pub fn cloudfront_distributions(&self) -> usize {
        self.0.iter()
            .filter(|&arn| arn.starts_with(CLOUDFRONT_OAI))
            .count()
    }

    pub fn wildcards(&self) -> usize {
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
                debug!("Working with object: {:?}", o);

                // This could also be "Federated", "Service", "CanonicalUser",
                // etc, but we aren't interested in those.
                if let Some(principal) = o.get("AWS") {
                    match principal {
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
                }
                else {
                    Self(Vec::new())
                }
            },
            _ => Self(Vec::new()),
        };

        output
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

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
        assert_eq!(principal.wildcards(), 1);
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
        assert_eq!(principal.wildcards(), 1);
    }
}
