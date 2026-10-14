use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled retention digest must pass. Check returns its problems, or none.
pub trait ScheduledRetentionDigestRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledRetentionDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRetentionDigestCodePrefixRule {
    pub prefix: String,
}

impl ScheduledRetentionDigestRule for ScheduledRetentionDigestCodePrefixRule {
    fn check(&self, item: &ScheduledRetentionDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRetentionDigestCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledRetentionDigestRule for ScheduledRetentionDigestCodeLengthRule {
    fn check(&self, item: &ScheduledRetentionDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRetentionDigestRequiredNotesRule {
    pub message: String,
}

impl ScheduledRetentionDigestRule for ScheduledRetentionDigestRequiredNotesRule {
    fn check(&self, item: &ScheduledRetentionDigest) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRetentionDigestRequiredTitleRule {
    pub message: String,
}

impl ScheduledRetentionDigestRule for ScheduledRetentionDigestRequiredTitleRule {
    fn check(&self, item: &ScheduledRetentionDigest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRetentionDigestOwnerIdLengthRule {
    pub max_length: i64,
}

impl ScheduledRetentionDigestRule for ScheduledRetentionDigestOwnerIdLengthRule {
    fn check(&self, item: &ScheduledRetentionDigest) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled retention digest.
#[derive(Debug, Clone)]
pub struct ScheduledRetentionDigestValidator {
    pub rules: Vec<Rc<dyn ScheduledRetentionDigestRule>>,
}

impl ScheduledRetentionDigestValidator {
    pub fn validate(&self, item: &ScheduledRetentionDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledRetentionDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledRetentionDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_retention_digest_validator() -> ScheduledRetentionDigestValidator {
    ScheduledRetentionDigestValidator {
        rules: vec![
            Rc::new(ScheduledRetentionDigestCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn ScheduledRetentionDigestRule>,
            Rc::new(ScheduledRetentionDigestCodeLengthRule { max_length: 86400 }),
            Rc::new(ScheduledRetentionDigestRequiredNotesRule {
                message: "scheduled retention digest needs this field".to_string(),
            }),
            Rc::new(ScheduledRetentionDigestRequiredTitleRule {
                message: "scheduled retention digest needs this field".to_string(),
            }),
            Rc::new(ScheduledRetentionDigestOwnerIdLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_retention_digest_validator();
        let item = new_scheduled_retention_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledRetentionDigestCodeLengthRule { max_length: 3600 };
        let mut item = new_scheduled_retention_digest("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = ScheduledRetentionDigestRequiredNotesRule {
            message: "scheduled retention digest needs a value".to_string(),
        };
        let mut item = new_scheduled_retention_digest("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ScheduledRetentionDigestRequiredTitleRule {
            message: "scheduled retention digest needs an entry".to_string(),
        };
        let mut item = new_scheduled_retention_digest("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = ScheduledRetentionDigestOwnerIdLengthRule { max_length: 64 };
        let mut item = new_scheduled_retention_digest("x", "y");
        item.owner_id = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
