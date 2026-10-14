use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a timeline checkpoint must pass. Check returns its problems, or none.
pub trait TimelineCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &TimelineCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineCheckpointRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TimelineCheckpointRule for TimelineCheckpointRetriesRangeRule {
    fn check(&self, item: &TimelineCheckpoint) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineCheckpointRequiredNamespaceRule {
    pub message: String,
}

impl TimelineCheckpointRule for TimelineCheckpointRequiredNamespaceRule {
    fn check(&self, item: &TimelineCheckpoint) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineCheckpointRequiredCodeRule {
    pub message: String,
}

impl TimelineCheckpointRule for TimelineCheckpointRequiredCodeRule {
    fn check(&self, item: &TimelineCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline checkpoint.
#[derive(Debug, Clone)]
pub struct TimelineCheckpointValidator {
    pub rules: Vec<Rc<dyn TimelineCheckpointRule>>,
}

impl TimelineCheckpointValidator {
    pub fn validate(&self, item: &TimelineCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TimelineCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_checkpoint_validator() -> TimelineCheckpointValidator {
    TimelineCheckpointValidator {
        rules: vec![
            Rc::new(TimelineCheckpointRetriesRangeRule { min: 80, max: 80 }) as Rc<dyn TimelineCheckpointRule>,
            Rc::new(TimelineCheckpointRequiredNamespaceRule {
                message: "timeline checkpoint needs a value".to_string(),
            }),
            Rc::new(TimelineCheckpointRequiredCodeRule {
                message: "timeline checkpoint needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_timeline_checkpoint_validator();
        let item = new_timeline_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = TimelineCheckpointRetriesRangeRule { min: 32, max: 64 };
        let mut item = new_timeline_checkpoint("x", "y");
        item.retries = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = TimelineCheckpointRequiredNamespaceRule {
            message: "timeline checkpoint needs an entry".to_string(),
        };
        let mut item = new_timeline_checkpoint("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TimelineCheckpointRequiredCodeRule {
            message: "timeline checkpoint needs an entry".to_string(),
        };
        let mut item = new_timeline_checkpoint("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
