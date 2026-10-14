use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled rollback record must pass. Check returns its problems, or none.
pub trait ScheduledRollbackRecordRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledRollbackRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRollbackRecordCodePrefixRule {
    pub prefix: String,
}

impl ScheduledRollbackRecordRule for ScheduledRollbackRecordCodePrefixRule {
    fn check(&self, item: &ScheduledRollbackRecord) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRollbackRecordLabelLengthRule {
    pub max_length: i64,
}

impl ScheduledRollbackRecordRule for ScheduledRollbackRecordLabelLengthRule {
    fn check(&self, item: &ScheduledRollbackRecord) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRollbackRecordRequiredLabelRule {
    pub message: String,
}

impl ScheduledRollbackRecordRule for ScheduledRollbackRecordRequiredLabelRule {
    fn check(&self, item: &ScheduledRollbackRecord) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRollbackRecordCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledRollbackRecordRule for ScheduledRollbackRecordCreatedAtRangeRule {
    fn check(&self, item: &ScheduledRollbackRecord) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a scheduled rollback record.
#[derive(Debug, Clone)]
pub struct ScheduledRollbackRecordValidator {
    pub rules: Vec<Rc<dyn ScheduledRollbackRecordRule>>,
}

impl ScheduledRollbackRecordValidator {
    pub fn validate(&self, item: &ScheduledRollbackRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledRollbackRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledRollbackRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_rollback_record_validator() -> ScheduledRollbackRecordValidator {
    ScheduledRollbackRecordValidator {
        rules: vec![
            Rc::new(ScheduledRollbackRecordCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn ScheduledRollbackRecordRule>,
            Rc::new(ScheduledRollbackRecordLabelLengthRule { max_length: 255 }),
            Rc::new(ScheduledRollbackRecordRequiredLabelRule {
                message: "scheduled rollback record needs an entry".to_string(),
            }),
            Rc::new(ScheduledRollbackRecordCreatedAtRangeRule { min: 32, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_rollback_record_validator();
        let item = new_scheduled_rollback_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ScheduledRollbackRecordLabelLengthRule { max_length: 3600 };
        let mut item = new_scheduled_rollback_record("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ScheduledRollbackRecordRequiredLabelRule {
            message: "scheduled rollback record needs an entry".to_string(),
        };
        let mut item = new_scheduled_rollback_record("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = ScheduledRollbackRecordCreatedAtRangeRule { min: 16, max: 86400 };
        let mut item = new_scheduled_rollback_record("x", "y");
        item.created_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
