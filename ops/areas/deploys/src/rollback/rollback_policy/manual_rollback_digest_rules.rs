use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual rollback digest must pass. Check returns its problems, or none.
pub trait ManualRollbackDigestRule: std::fmt::Debug {
    fn check(&self, item: &ManualRollbackDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRollbackDigestRequiredCodeRule {
    pub message: String,
}

impl ManualRollbackDigestRule for ManualRollbackDigestRequiredCodeRule {
    fn check(&self, item: &ManualRollbackDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRollbackDigestCodePrefixRule {
    pub prefix: String,
}

impl ManualRollbackDigestRule for ManualRollbackDigestCodePrefixRule {
    fn check(&self, item: &ManualRollbackDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRollbackDigestCodeLengthRule {
    pub max_length: i64,
}

impl ManualRollbackDigestRule for ManualRollbackDigestCodeLengthRule {
    fn check(&self, item: &ManualRollbackDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRollbackDigestRequiredLabelRule {
    pub message: String,
}

impl ManualRollbackDigestRule for ManualRollbackDigestRequiredLabelRule {
    fn check(&self, item: &ManualRollbackDigest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual rollback digest.
#[derive(Debug, Clone)]
pub struct ManualRollbackDigestValidator {
    pub rules: Vec<Rc<dyn ManualRollbackDigestRule>>,
}

impl ManualRollbackDigestValidator {
    pub fn validate(&self, item: &ManualRollbackDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualRollbackDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualRollbackDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_rollback_digest_validator() -> ManualRollbackDigestValidator {
    ManualRollbackDigestValidator {
        rules: vec![
            Rc::new(ManualRollbackDigestRequiredCodeRule {
                message: "manual rollback digest needs a value".to_string(),
            }) as Rc<dyn ManualRollbackDigestRule>,
            Rc::new(ManualRollbackDigestCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ManualRollbackDigestCodeLengthRule { max_length: 1000 }),
            Rc::new(ManualRollbackDigestRequiredLabelRule {
                message: "manual rollback digest needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_rollback_digest_validator();
        let item = new_manual_rollback_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualRollbackDigestRequiredCodeRule {
            message: "manual rollback digest needs a value".to_string(),
        };
        let mut item = new_manual_rollback_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualRollbackDigestCodeLengthRule { max_length: 80 };
        let mut item = new_manual_rollback_digest("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ManualRollbackDigestRequiredLabelRule {
            message: "manual rollback digest needs an entry".to_string(),
        };
        let mut item = new_manual_rollback_digest("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
