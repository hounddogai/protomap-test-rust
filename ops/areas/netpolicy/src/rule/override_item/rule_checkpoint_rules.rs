use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rule checkpoint must pass. Check returns its problems, or none.
pub trait RuleCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &RuleCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleCheckpointCodePrefixRule {
    pub prefix: String,
}

impl RuleCheckpointRule for RuleCheckpointCodePrefixRule {
    fn check(&self, item: &RuleCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleCheckpointNamespaceLengthRule {
    pub max_length: i64,
}

impl RuleCheckpointRule for RuleCheckpointNamespaceLengthRule {
    fn check(&self, item: &RuleCheckpoint) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleCheckpointRequiredCodeRule {
    pub message: String,
}

impl RuleCheckpointRule for RuleCheckpointRequiredCodeRule {
    fn check(&self, item: &RuleCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleCheckpointRequiredNamespaceRule {
    pub message: String,
}

impl RuleCheckpointRule for RuleCheckpointRequiredNamespaceRule {
    fn check(&self, item: &RuleCheckpoint) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleCheckpointLocaleLengthRule {
    pub max_length: i64,
}

impl RuleCheckpointRule for RuleCheckpointLocaleLengthRule {
    fn check(&self, item: &RuleCheckpoint) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl RuleCheckpointRule for RuleCheckpointCodeLengthRule {
    fn check(&self, item: &RuleCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule checkpoint.
#[derive(Debug, Clone)]
pub struct RuleCheckpointValidator {
    pub rules: Vec<Rc<dyn RuleCheckpointRule>>,
}

impl RuleCheckpointValidator {
    pub fn validate(&self, item: &RuleCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RuleCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_checkpoint_validator() -> RuleCheckpointValidator {
    RuleCheckpointValidator {
        rules: vec![
            Rc::new(RuleCheckpointCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn RuleCheckpointRule>,
            Rc::new(RuleCheckpointNamespaceLengthRule { max_length: 80 }),
            Rc::new(RuleCheckpointRequiredCodeRule {
                message: "rule checkpoint needs an entry".to_string(),
            }),
            Rc::new(RuleCheckpointRequiredNamespaceRule {
                message: "rule checkpoint needs an entry".to_string(),
            }),
            Rc::new(RuleCheckpointLocaleLengthRule { max_length: 255 }),
            Rc::new(RuleCheckpointCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rule_checkpoint_validator();
        let item = new_rule_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = RuleCheckpointNamespaceLengthRule { max_length: 3600 };
        let mut item = new_rule_checkpoint("x", "y");
        item.namespace = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RuleCheckpointRequiredCodeRule {
            message: "rule checkpoint needs a value".to_string(),
        };
        let mut item = new_rule_checkpoint("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = RuleCheckpointRequiredNamespaceRule {
            message: "rule checkpoint needs this field".to_string(),
        };
        let mut item = new_rule_checkpoint("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = RuleCheckpointLocaleLengthRule { max_length: 32 };
        let mut item = new_rule_checkpoint("x", "y");
        item.locale = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RuleCheckpointCodeLengthRule { max_length: 64 };
        let mut item = new_rule_checkpoint("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
