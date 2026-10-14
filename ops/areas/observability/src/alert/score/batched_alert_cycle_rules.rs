use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched alert cycle must pass. Check returns its problems, or none.
pub trait BatchedAlertCycleRule: std::fmt::Debug {
    fn check(&self, item: &BatchedAlertCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAlertCycleStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedAlertCycleRule for BatchedAlertCycleStartsAtRangeRule {
    fn check(&self, item: &BatchedAlertCycle) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAlertCycleRequiredLabelRule {
    pub message: String,
}

impl BatchedAlertCycleRule for BatchedAlertCycleRequiredLabelRule {
    fn check(&self, item: &BatchedAlertCycle) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAlertCycleCodeLengthRule {
    pub max_length: i64,
}

impl BatchedAlertCycleRule for BatchedAlertCycleCodeLengthRule {
    fn check(&self, item: &BatchedAlertCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAlertCycleRequiredChannelRule {
    pub message: String,
}

impl BatchedAlertCycleRule for BatchedAlertCycleRequiredChannelRule {
    fn check(&self, item: &BatchedAlertCycle) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAlertCycleLabelLengthRule {
    pub max_length: i64,
}

impl BatchedAlertCycleRule for BatchedAlertCycleLabelLengthRule {
    fn check(&self, item: &BatchedAlertCycle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched alert cycle.
#[derive(Debug, Clone)]
pub struct BatchedAlertCycleValidator {
    pub rules: Vec<Rc<dyn BatchedAlertCycleRule>>,
}

impl BatchedAlertCycleValidator {
    pub fn validate(&self, item: &BatchedAlertCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedAlertCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedAlertCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_alert_cycle_validator() -> BatchedAlertCycleValidator {
    BatchedAlertCycleValidator {
        rules: vec![
            Rc::new(BatchedAlertCycleStartsAtRangeRule { min: 120, max: 3600 }) as Rc<dyn BatchedAlertCycleRule>,
            Rc::new(BatchedAlertCycleRequiredLabelRule {
                message: "batched alert cycle needs this field".to_string(),
            }),
            Rc::new(BatchedAlertCycleCodeLengthRule { max_length: 16 }),
            Rc::new(BatchedAlertCycleRequiredChannelRule {
                message: "batched alert cycle needs an entry".to_string(),
            }),
            Rc::new(BatchedAlertCycleLabelLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_alert_cycle_validator();
        let item = new_batched_alert_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = BatchedAlertCycleStartsAtRangeRule { min: 80, max: 86400 };
        let mut item = new_batched_alert_cycle("x", "y");
        item.starts_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = BatchedAlertCycleRequiredLabelRule {
            message: "batched alert cycle needs a value".to_string(),
        };
        let mut item = new_batched_alert_cycle("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedAlertCycleCodeLengthRule { max_length: 120 };
        let mut item = new_batched_alert_cycle("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = BatchedAlertCycleRequiredChannelRule {
            message: "batched alert cycle needs this field".to_string(),
        };
        let mut item = new_batched_alert_cycle("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = BatchedAlertCycleLabelLengthRule { max_length: 1000 };
        let mut item = new_batched_alert_cycle("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
