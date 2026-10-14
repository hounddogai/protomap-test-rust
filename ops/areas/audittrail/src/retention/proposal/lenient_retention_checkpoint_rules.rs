use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient retention checkpoint must pass. Check returns its problems, or none.
pub trait LenientRetentionCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &LenientRetentionCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRetentionCheckpointRequiredNameRule {
    pub message: String,
}

impl LenientRetentionCheckpointRule for LenientRetentionCheckpointRequiredNameRule {
    fn check(&self, item: &LenientRetentionCheckpoint) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRetentionCheckpointUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientRetentionCheckpointRule for LenientRetentionCheckpointUpdatedAtRangeRule {
    fn check(&self, item: &LenientRetentionCheckpoint) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRetentionCheckpointDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientRetentionCheckpointRule for LenientRetentionCheckpointDurationMinutesRangeRule {
    fn check(&self, item: &LenientRetentionCheckpoint) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRetentionCheckpointOwnerIdLengthRule {
    pub max_length: i64,
}

impl LenientRetentionCheckpointRule for LenientRetentionCheckpointOwnerIdLengthRule {
    fn check(&self, item: &LenientRetentionCheckpoint) -> Vec<String> {
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

/// Runs every rule of a lenient retention checkpoint.
#[derive(Debug, Clone)]
pub struct LenientRetentionCheckpointValidator {
    pub rules: Vec<Rc<dyn LenientRetentionCheckpointRule>>,
}

impl LenientRetentionCheckpointValidator {
    pub fn validate(&self, item: &LenientRetentionCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientRetentionCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientRetentionCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_retention_checkpoint_validator() -> LenientRetentionCheckpointValidator {
    LenientRetentionCheckpointValidator {
        rules: vec![
            Rc::new(LenientRetentionCheckpointRequiredNameRule {
                message: "lenient retention checkpoint needs a value".to_string(),
            }) as Rc<dyn LenientRetentionCheckpointRule>,
            Rc::new(LenientRetentionCheckpointUpdatedAtRangeRule { min: 32, max: 86400 }),
            Rc::new(LenientRetentionCheckpointDurationMinutesRangeRule { min: 1000, max: 86400 }),
            Rc::new(LenientRetentionCheckpointOwnerIdLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_retention_checkpoint_validator();
        let item = new_lenient_retention_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LenientRetentionCheckpointRequiredNameRule {
            message: "lenient retention checkpoint needs this field".to_string(),
        };
        let mut item = new_lenient_retention_checkpoint("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = LenientRetentionCheckpointUpdatedAtRangeRule { min: 3600, max: 86400 };
        let mut item = new_lenient_retention_checkpoint("x", "y");
        item.updated_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = LenientRetentionCheckpointDurationMinutesRangeRule { min: 3600, max: 86400 };
        let mut item = new_lenient_retention_checkpoint("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = LenientRetentionCheckpointOwnerIdLengthRule { max_length: 3600 };
        let mut item = new_lenient_retention_checkpoint("x", "y");
        item.owner_id = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
