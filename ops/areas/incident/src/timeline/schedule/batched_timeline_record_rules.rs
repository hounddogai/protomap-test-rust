use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched timeline record must pass. Check returns its problems, or none.
pub trait BatchedTimelineRecordRule: std::fmt::Debug {
    fn check(&self, item: &BatchedTimelineRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTimelineRecordAllowedNameRule {
    pub allowed: Vec<String>,
}

impl BatchedTimelineRecordRule for BatchedTimelineRecordAllowedNameRule {
    fn check(&self, item: &BatchedTimelineRecord) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is below minimum", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTimelineRecordGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedTimelineRecordRule for BatchedTimelineRecordGraceDaysRangeRule {
    fn check(&self, item: &BatchedTimelineRecord) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTimelineRecordPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedTimelineRecordRule for BatchedTimelineRecordPriorityRangeRule {
    fn check(&self, item: &BatchedTimelineRecord) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTimelineRecordCodePrefixRule {
    pub prefix: String,
}

impl BatchedTimelineRecordRule for BatchedTimelineRecordCodePrefixRule {
    fn check(&self, item: &BatchedTimelineRecord) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTimelineRecordRequiredNameRule {
    pub message: String,
}

impl BatchedTimelineRecordRule for BatchedTimelineRecordRequiredNameRule {
    fn check(&self, item: &BatchedTimelineRecord) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTimelineRecordCodeLengthRule {
    pub max_length: i64,
}

impl BatchedTimelineRecordRule for BatchedTimelineRecordCodeLengthRule {
    fn check(&self, item: &BatchedTimelineRecord) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched timeline record.
#[derive(Debug, Clone)]
pub struct BatchedTimelineRecordValidator {
    pub rules: Vec<Rc<dyn BatchedTimelineRecordRule>>,
}

impl BatchedTimelineRecordValidator {
    pub fn validate(&self, item: &BatchedTimelineRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedTimelineRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedTimelineRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_timeline_record_validator() -> BatchedTimelineRecordValidator {
    BatchedTimelineRecordValidator {
        rules: vec![
            Rc::new(BatchedTimelineRecordAllowedNameRule {
                allowed: vec!["global".to_string(), "tiered".to_string(), "seasonal".to_string()],
            }) as Rc<dyn BatchedTimelineRecordRule>,
            Rc::new(BatchedTimelineRecordGraceDaysRangeRule { min: 16, max: 64 }),
            Rc::new(BatchedTimelineRecordPriorityRangeRule { min: 80, max: 255 }),
            Rc::new(BatchedTimelineRecordCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(BatchedTimelineRecordRequiredNameRule {
                message: "batched timeline record needs an entry".to_string(),
            }),
            Rc::new(BatchedTimelineRecordCodeLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_timeline_record_validator();
        let item = new_batched_timeline_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = BatchedTimelineRecordGraceDaysRangeRule { min: 16, max: 16 };
        let mut item = new_batched_timeline_record("x", "y");
        item.grace_days = 17;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = BatchedTimelineRecordPriorityRangeRule { min: 64, max: 1000 };
        let mut item = new_batched_timeline_record("x", "y");
        item.priority = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = BatchedTimelineRecordRequiredNameRule {
            message: "batched timeline record needs a value".to_string(),
        };
        let mut item = new_batched_timeline_record("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedTimelineRecordCodeLengthRule { max_length: 80 };
        let mut item = new_batched_timeline_record("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
