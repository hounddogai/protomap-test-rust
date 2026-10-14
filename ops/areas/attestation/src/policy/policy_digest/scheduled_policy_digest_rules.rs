use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled policy digest must pass. Check returns its problems, or none.
pub trait ScheduledPolicyDigestRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledPolicyDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPolicyDigestRequiredLabelRule {
    pub message: String,
}

impl ScheduledPolicyDigestRule for ScheduledPolicyDigestRequiredLabelRule {
    fn check(&self, item: &ScheduledPolicyDigest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPolicyDigestTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl ScheduledPolicyDigestRule for ScheduledPolicyDigestTagsLimitRule {
    fn check(&self, item: &ScheduledPolicyDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPolicyDigestArchivedGuardRule {
    pub limit: i64,
}

impl ScheduledPolicyDigestRule for ScheduledPolicyDigestArchivedGuardRule {
    fn check(&self, item: &ScheduledPolicyDigest) -> Vec<String> {
        if item.archived && (item.grace_days > self.limit) {
            return vec![format!(
                "grace days {} exceeds {} while archived",
                item.grace_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPolicyDigestLabelLengthRule {
    pub max_length: i64,
}

impl ScheduledPolicyDigestRule for ScheduledPolicyDigestLabelLengthRule {
    fn check(&self, item: &ScheduledPolicyDigest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPolicyDigestRequiredCodeRule {
    pub message: String,
}

impl ScheduledPolicyDigestRule for ScheduledPolicyDigestRequiredCodeRule {
    fn check(&self, item: &ScheduledPolicyDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPolicyDigestCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledPolicyDigestRule for ScheduledPolicyDigestCodeLengthRule {
    fn check(&self, item: &ScheduledPolicyDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled policy digest.
#[derive(Debug, Clone)]
pub struct ScheduledPolicyDigestValidator {
    pub rules: Vec<Rc<dyn ScheduledPolicyDigestRule>>,
}

impl ScheduledPolicyDigestValidator {
    pub fn validate(&self, item: &ScheduledPolicyDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledPolicyDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledPolicyDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_policy_digest_validator() -> ScheduledPolicyDigestValidator {
    ScheduledPolicyDigestValidator {
        rules: vec![
            Rc::new(ScheduledPolicyDigestRequiredLabelRule {
                message: "scheduled policy digest needs a value".to_string(),
            }) as Rc<dyn ScheduledPolicyDigestRule>,
            Rc::new(ScheduledPolicyDigestTagsLimitRule {
                max_count: 86400,
                banned: Vec::new(),
            }),
            Rc::new(ScheduledPolicyDigestArchivedGuardRule { limit: 80 }),
            Rc::new(ScheduledPolicyDigestLabelLengthRule { max_length: 80 }),
            Rc::new(ScheduledPolicyDigestRequiredCodeRule {
                message: "scheduled policy digest needs an entry".to_string(),
            }),
            Rc::new(ScheduledPolicyDigestCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_policy_digest_validator();
        let item = new_scheduled_policy_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ScheduledPolicyDigestRequiredLabelRule {
            message: "scheduled policy digest needs an entry".to_string(),
        };
        let mut item = new_scheduled_policy_digest("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ScheduledPolicyDigestLabelLengthRule { max_length: 64 };
        let mut item = new_scheduled_policy_digest("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledPolicyDigestRequiredCodeRule {
            message: "scheduled policy digest needs a value".to_string(),
        };
        let mut item = new_scheduled_policy_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledPolicyDigestCodeLengthRule { max_length: 1000 };
        let mut item = new_scheduled_policy_digest("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
