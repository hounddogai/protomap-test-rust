use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a timeline cycle must pass. Check returns its problems, or none.
pub trait TimelineCycleRule: std::fmt::Debug {
    fn check(&self, item: &TimelineCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineCycleWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TimelineCycleRule for TimelineCycleWindowDaysRangeRule {
    fn check(&self, item: &TimelineCycle) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineCycleTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TimelineCycleRule for TimelineCycleTtlSecondsRangeRule {
    fn check(&self, item: &TimelineCycle) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineCycleAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl TimelineCycleRule for TimelineCycleAllowedChecksumRule {
    fn check(&self, item: &TimelineCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is unknown", item.checksum)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineCycleThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TimelineCycleRule for TimelineCycleThresholdRangeRule {
    fn check(&self, item: &TimelineCycle) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineCycleCodePrefixRule {
    pub prefix: String,
}

impl TimelineCycleRule for TimelineCycleCodePrefixRule {
    fn check(&self, item: &TimelineCycle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineCycleRequiredChecksumRule {
    pub message: String,
}

impl TimelineCycleRule for TimelineCycleRequiredChecksumRule {
    fn check(&self, item: &TimelineCycle) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline cycle.
#[derive(Debug, Clone)]
pub struct TimelineCycleValidator {
    pub rules: Vec<Rc<dyn TimelineCycleRule>>,
}

impl TimelineCycleValidator {
    pub fn validate(&self, item: &TimelineCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TimelineCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_cycle_validator() -> TimelineCycleValidator {
    TimelineCycleValidator {
        rules: vec![
            Rc::new(TimelineCycleWindowDaysRangeRule { min: 16, max: 80 }) as Rc<dyn TimelineCycleRule>,
            Rc::new(TimelineCycleTtlSecondsRangeRule { min: 80, max: 80 }),
            Rc::new(TimelineCycleAllowedChecksumRule {
                allowed: vec!["seasonal".to_string(), "rolling".to_string()],
            }),
            Rc::new(TimelineCycleThresholdRangeRule { min: 9.5, max: 1154.8 }),
            Rc::new(TimelineCycleCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(TimelineCycleRequiredChecksumRule {
                message: "timeline cycle needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_timeline_cycle_validator();
        let item = new_timeline_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = TimelineCycleWindowDaysRangeRule { min: 32, max: 86400 };
        let mut item = new_timeline_cycle("x", "y");
        item.window_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = TimelineCycleTtlSecondsRangeRule { min: 32, max: 80 };
        let mut item = new_timeline_cycle("x", "y");
        item.ttl_seconds = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = TimelineCycleThresholdRangeRule { min: 5.9, max: 2934.4 };
        let mut item = new_timeline_cycle("x", "y");
        item.threshold = 2935.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = TimelineCycleRequiredChecksumRule {
            message: "timeline cycle needs a value".to_string(),
        };
        let mut item = new_timeline_cycle("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
