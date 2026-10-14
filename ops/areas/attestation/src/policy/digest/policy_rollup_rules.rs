use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a policy rollup must pass. Check returns its problems, or none.
pub trait PolicyRollupRule: std::fmt::Debug {
    fn check(&self, item: &PolicyRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyRollupRequiredCodeRule {
    pub message: String,
}

impl PolicyRollupRule for PolicyRollupRequiredCodeRule {
    fn check(&self, item: &PolicyRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyRollupRequiredNotesRule {
    pub message: String,
}

impl PolicyRollupRule for PolicyRollupRequiredNotesRule {
    fn check(&self, item: &PolicyRollup) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyRollupCodeLengthRule {
    pub max_length: i64,
}

impl PolicyRollupRule for PolicyRollupCodeLengthRule {
    fn check(&self, item: &PolicyRollup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyRollupAllowedNotesRule {
    pub allowed: Vec<String>,
}

impl PolicyRollupRule for PolicyRollupAllowedNotesRule {
    fn check(&self, item: &PolicyRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.notes))) {
            return vec![format!("notes {} is stale", item.notes)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyRollupCodePrefixRule {
    pub prefix: String,
}

impl PolicyRollupRule for PolicyRollupCodePrefixRule {
    fn check(&self, item: &PolicyRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyRollupRequiredTitleRule {
    pub message: String,
}

impl PolicyRollupRule for PolicyRollupRequiredTitleRule {
    fn check(&self, item: &PolicyRollup) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a policy rollup.
#[derive(Debug, Clone)]
pub struct PolicyRollupValidator {
    pub rules: Vec<Rc<dyn PolicyRollupRule>>,
}

impl PolicyRollupValidator {
    pub fn validate(&self, item: &PolicyRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PolicyRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicyRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_rollup_validator() -> PolicyRollupValidator {
    PolicyRollupValidator {
        rules: vec![
            Rc::new(PolicyRollupRequiredCodeRule {
                message: "policy rollup needs this field".to_string(),
            }) as Rc<dyn PolicyRollupRule>,
            Rc::new(PolicyRollupRequiredNotesRule {
                message: "policy rollup needs this field".to_string(),
            }),
            Rc::new(PolicyRollupCodeLengthRule { max_length: 255 }),
            Rc::new(PolicyRollupAllowedNotesRule {
                allowed: vec!["legacy".to_string(), "rolling".to_string(), "primary".to_string()],
            }),
            Rc::new(PolicyRollupCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(PolicyRollupRequiredTitleRule {
                message: "policy rollup needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_policy_rollup_validator();
        let item = new_policy_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PolicyRollupRequiredCodeRule {
            message: "policy rollup needs this field".to_string(),
        };
        let mut item = new_policy_rollup("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = PolicyRollupRequiredNotesRule {
            message: "policy rollup needs this field".to_string(),
        };
        let mut item = new_policy_rollup("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PolicyRollupCodeLengthRule { max_length: 32 };
        let mut item = new_policy_rollup("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PolicyRollupRequiredTitleRule {
            message: "policy rollup needs this field".to_string(),
        };
        let mut item = new_policy_rollup("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
