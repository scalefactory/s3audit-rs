// Checks if S3 policies allow wildcard entities on Actions
use serde_json::Value;

const WILDCARD: &str = "*";

#[derive(Debug, Default)]
pub struct Action(Vec<String>);

impl Action {
    pub fn append(&mut self, mut other: Self) {
        self.0.append(&mut other.0);
    }

    pub fn wildcards(&self) -> usize {
        // Wildcards could appear anywhere in the name
        // eg. "*", "s3:*", "iam:*AccessKey*"
        self.0.iter()
            .filter(|&name| name.contains(WILDCARD))
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

// Takes a Value representing the Action entry in a Bucket Policy and
// returns a Vec of the discovered names wrapped in an Action struct.
impl From<&Value> for Action {
    fn from(value: &Value) -> Self {
        let output = match value {
            // "Action": "s3:Foo"
            Value::String(action) => {
                let actions = vec![
                    String::from(action),
                ];

                Self(actions)
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
}
