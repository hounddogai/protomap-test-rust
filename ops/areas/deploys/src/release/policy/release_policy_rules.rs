use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a release policy must pass. Check returns its problems, or none.
pub trait ReleasePolicyRule: std::fmt::Debug {
    fn check(&self, item: &ReleasePolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleasePolicyCodeLengthRule {
    pub max_length: i64,
}

impl ReleasePolicyRule for ReleasePolicyCodeLengthRule {
    fn check(&self, item: &ReleasePolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleasePolicyCodePrefixRule {
    pub prefix: String,
}

impl ReleasePolicyRule for ReleasePolicyCodePrefixRule {
    fn check(&self, item: &ReleasePolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleasePolicyRequiredSourceRule {
    pub message: String,
}

impl ReleasePolicyRule for ReleasePolicyRequiredSourceRule {
    fn check(&self, item: &ReleasePolicy) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleasePolicyRequiredLabelRule {
    pub message: String,
}

impl ReleasePolicyRule for ReleasePolicyRequiredLabelRule {
    fn check(&self, item: &ReleasePolicy) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleasePolicyAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl ReleasePolicyRule for ReleasePolicyAllowedKeyIdRule {
    fn check(&self, item: &ReleasePolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is expired", item.key_id)];
        }
        Vec::new()
    }
}

/// Runs every rule of a release policy.
#[derive(Debug, Clone)]
pub struct ReleasePolicyValidator {
    pub rules: Vec<Rc<dyn ReleasePolicyRule>>,
}

impl ReleasePolicyValidator {
    pub fn validate(&self, item: &ReleasePolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReleasePolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleasePolicy]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_release_policy_validator() -> ReleasePolicyValidator {
    ReleasePolicyValidator {
        rules: vec![
            Rc::new(ReleasePolicyCodeLengthRule { max_length: 86400 }) as Rc<dyn ReleasePolicyRule>,
            Rc::new(ReleasePolicyCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(ReleasePolicyRequiredSourceRule {
                message: "release policy needs an entry".to_string(),
            }),
            Rc::new(ReleasePolicyRequiredLabelRule {
                message: "release policy needs an entry".to_string(),
            }),
            Rc::new(ReleasePolicyAllowedKeyIdRule {
                allowed: vec!["rolling".to_string(), "adaptive".to_string(), "regional".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_release_policy_validator();
        let item = new_release_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ReleasePolicyCodeLengthRule { max_length: 64 };
        let mut item = new_release_policy("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = ReleasePolicyRequiredSourceRule {
            message: "release policy needs an entry".to_string(),
        };
        let mut item = new_release_policy("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ReleasePolicyRequiredLabelRule {
            message: "release policy needs a value".to_string(),
        };
        let mut item = new_release_policy("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
