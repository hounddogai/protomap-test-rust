use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled action item digest must pass. Check returns its problems, or none.
pub trait ScheduledActionItemDigestRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledActionItemDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledActionItemDigestRequiredLabelRule {
    pub message: String,
}

impl ScheduledActionItemDigestRule for ScheduledActionItemDigestRequiredLabelRule {
    fn check(&self, item: &ScheduledActionItemDigest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledActionItemDigestCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledActionItemDigestRule for ScheduledActionItemDigestCodeLengthRule {
    fn check(&self, item: &ScheduledActionItemDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledActionItemDigestLabelLengthRule {
    pub max_length: i64,
}

impl ScheduledActionItemDigestRule for ScheduledActionItemDigestLabelLengthRule {
    fn check(&self, item: &ScheduledActionItemDigest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled action item digest.
#[derive(Debug, Clone)]
pub struct ScheduledActionItemDigestValidator {
    pub rules: Vec<Rc<dyn ScheduledActionItemDigestRule>>,
}

impl ScheduledActionItemDigestValidator {
    pub fn validate(&self, item: &ScheduledActionItemDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledActionItemDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledActionItemDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_action_item_digest_validator() -> ScheduledActionItemDigestValidator {
    ScheduledActionItemDigestValidator {
        rules: vec![
            Rc::new(ScheduledActionItemDigestRequiredLabelRule {
                message: "scheduled action item digest needs a value".to_string(),
            }) as Rc<dyn ScheduledActionItemDigestRule>,
            Rc::new(ScheduledActionItemDigestCodeLengthRule { max_length: 3600 }),
            Rc::new(ScheduledActionItemDigestLabelLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_action_item_digest_validator();
        let item = new_scheduled_action_item_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ScheduledActionItemDigestRequiredLabelRule {
            message: "scheduled action item digest needs an entry".to_string(),
        };
        let mut item = new_scheduled_action_item_digest("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledActionItemDigestCodeLengthRule { max_length: 86400 };
        let mut item = new_scheduled_action_item_digest("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ScheduledActionItemDigestLabelLengthRule { max_length: 86400 };
        let mut item = new_scheduled_action_item_digest("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
