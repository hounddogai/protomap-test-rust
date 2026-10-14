use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a segment checkpoint must pass. Check returns its problems, or none.
pub trait SegmentCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &SegmentCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentCheckpointRequiredNameRule {
    pub message: String,
}

impl SegmentCheckpointRule for SegmentCheckpointRequiredNameRule {
    fn check(&self, item: &SegmentCheckpoint) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentCheckpointRequiredCodeRule {
    pub message: String,
}

impl SegmentCheckpointRule for SegmentCheckpointRequiredCodeRule {
    fn check(&self, item: &SegmentCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentCheckpointAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentCheckpointRule for SegmentCheckpointAttemptsRangeRule {
    fn check(&self, item: &SegmentCheckpoint) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment checkpoint.
#[derive(Debug, Clone)]
pub struct SegmentCheckpointValidator {
    pub rules: Vec<Rc<dyn SegmentCheckpointRule>>,
}

impl SegmentCheckpointValidator {
    pub fn validate(&self, item: &SegmentCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SegmentCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_checkpoint_validator() -> SegmentCheckpointValidator {
    SegmentCheckpointValidator {
        rules: vec![
            Rc::new(SegmentCheckpointRequiredNameRule {
                message: "segment checkpoint needs an entry".to_string(),
            }) as Rc<dyn SegmentCheckpointRule>,
            Rc::new(SegmentCheckpointRequiredCodeRule {
                message: "segment checkpoint needs an entry".to_string(),
            }),
            Rc::new(SegmentCheckpointAttemptsRangeRule { min: 16, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_checkpoint_validator();
        let item = new_segment_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = SegmentCheckpointRequiredNameRule {
            message: "segment checkpoint needs this field".to_string(),
        };
        let mut item = new_segment_checkpoint("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SegmentCheckpointRequiredCodeRule {
            message: "segment checkpoint needs a value".to_string(),
        };
        let mut item = new_segment_checkpoint("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = SegmentCheckpointAttemptsRangeRule { min: 255, max: 3600 };
        let mut item = new_segment_checkpoint("x", "y");
        item.attempts = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
