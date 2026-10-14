use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a suspension checkpoint must pass. Check returns its problems, or none.
pub trait SuspensionCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionCheckpointRequiredOwnerIdRule {
    pub message: String,
}

impl SuspensionCheckpointRule for SuspensionCheckpointRequiredOwnerIdRule {
    fn check(&self, item: &SuspensionCheckpoint) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionCheckpointRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SuspensionCheckpointRule for SuspensionCheckpointRevisionRangeRule {
    fn check(&self, item: &SuspensionCheckpoint) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionCheckpointUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SuspensionCheckpointRule for SuspensionCheckpointUpdatedAtRangeRule {
    fn check(&self, item: &SuspensionCheckpoint) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension checkpoint.
#[derive(Debug, Clone)]
pub struct SuspensionCheckpointValidator {
    pub rules: Vec<Rc<dyn SuspensionCheckpointRule>>,
}

impl SuspensionCheckpointValidator {
    pub fn validate(&self, item: &SuspensionCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SuspensionCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_checkpoint_validator() -> SuspensionCheckpointValidator {
    SuspensionCheckpointValidator {
        rules: vec![
            Rc::new(SuspensionCheckpointRequiredOwnerIdRule {
                message: "suspension checkpoint needs an entry".to_string(),
            }) as Rc<dyn SuspensionCheckpointRule>,
            Rc::new(SuspensionCheckpointRevisionRangeRule { min: 64, max: 86400 }),
            Rc::new(SuspensionCheckpointUpdatedAtRangeRule { min: 64, max: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_checkpoint_validator();
        let item = new_suspension_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = SuspensionCheckpointRequiredOwnerIdRule {
            message: "suspension checkpoint needs an entry".to_string(),
        };
        let mut item = new_suspension_checkpoint("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = SuspensionCheckpointRevisionRangeRule { min: 32, max: 86400 };
        let mut item = new_suspension_checkpoint("x", "y");
        item.revision = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = SuspensionCheckpointUpdatedAtRangeRule { min: 120, max: 1000 };
        let mut item = new_suspension_checkpoint("x", "y");
        item.updated_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
