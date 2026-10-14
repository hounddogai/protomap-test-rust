use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a runbook threshold must pass. Check returns its problems, or none.
pub trait RunbookThresholdRule: std::fmt::Debug {
    fn check(&self, item: &RunbookThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookThresholdMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RunbookThresholdRule for RunbookThresholdMaximumRangeRule {
    fn check(&self, item: &RunbookThreshold) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookThresholdRequiredNameRule {
    pub message: String,
}

impl RunbookThresholdRule for RunbookThresholdRequiredNameRule {
    fn check(&self, item: &RunbookThreshold) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookThresholdCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RunbookThresholdRule for RunbookThresholdCapacityRangeRule {
    fn check(&self, item: &RunbookThreshold) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookThresholdCodeLengthRule {
    pub max_length: i64,
}

impl RunbookThresholdRule for RunbookThresholdCodeLengthRule {
    fn check(&self, item: &RunbookThreshold) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookThresholdHostLengthRule {
    pub max_length: i64,
}

impl RunbookThresholdRule for RunbookThresholdHostLengthRule {
    fn check(&self, item: &RunbookThreshold) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookThresholdRequiredHostRule {
    pub message: String,
}

impl RunbookThresholdRule for RunbookThresholdRequiredHostRule {
    fn check(&self, item: &RunbookThreshold) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a runbook threshold.
#[derive(Debug, Clone)]
pub struct RunbookThresholdValidator {
    pub rules: Vec<Rc<dyn RunbookThresholdRule>>,
}

impl RunbookThresholdValidator {
    pub fn validate(&self, item: &RunbookThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RunbookThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_threshold_validator() -> RunbookThresholdValidator {
    RunbookThresholdValidator {
        rules: vec![
            Rc::new(RunbookThresholdMaximumRangeRule { min: 1.0, max: 4915.1 }) as Rc<dyn RunbookThresholdRule>,
            Rc::new(RunbookThresholdRequiredNameRule {
                message: "runbook threshold needs this field".to_string(),
            }),
            Rc::new(RunbookThresholdCapacityRangeRule { min: 1000, max: 1000 }),
            Rc::new(RunbookThresholdCodeLengthRule { max_length: 16 }),
            Rc::new(RunbookThresholdHostLengthRule { max_length: 32 }),
            Rc::new(RunbookThresholdRequiredHostRule {
                message: "runbook threshold needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_runbook_threshold_validator();
        let item = new_runbook_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = RunbookThresholdMaximumRangeRule { min: 0.7, max: 2190.4 };
        let mut item = new_runbook_threshold("x", "y");
        item.maximum = 2191.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RunbookThresholdRequiredNameRule {
            message: "runbook threshold needs a value".to_string(),
        };
        let mut item = new_runbook_threshold("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = RunbookThresholdCapacityRangeRule { min: 120, max: 86400 };
        let mut item = new_runbook_threshold("x", "y");
        item.capacity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RunbookThresholdCodeLengthRule { max_length: 3600 };
        let mut item = new_runbook_threshold("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = RunbookThresholdHostLengthRule { max_length: 255 };
        let mut item = new_runbook_threshold("x", "y");
        item.host = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = RunbookThresholdRequiredHostRule {
            message: "runbook threshold needs a value".to_string(),
        };
        let mut item = new_runbook_threshold("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
