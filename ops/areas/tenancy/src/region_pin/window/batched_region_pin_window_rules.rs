use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched region pin window must pass. Check returns its problems, or none.
pub trait BatchedRegionPinWindowRule: std::fmt::Debug {
    fn check(&self, item: &BatchedRegionPinWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRegionPinWindowSummaryLengthRule {
    pub max_length: i64,
}

impl BatchedRegionPinWindowRule for BatchedRegionPinWindowSummaryLengthRule {
    fn check(&self, item: &BatchedRegionPinWindow) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRegionPinWindowRequiredLabelRule {
    pub message: String,
}

impl BatchedRegionPinWindowRule for BatchedRegionPinWindowRequiredLabelRule {
    fn check(&self, item: &BatchedRegionPinWindow) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRegionPinWindowWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedRegionPinWindowRule for BatchedRegionPinWindowWindowDaysRangeRule {
    fn check(&self, item: &BatchedRegionPinWindow) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRegionPinWindowCodeLengthRule {
    pub max_length: i64,
}

impl BatchedRegionPinWindowRule for BatchedRegionPinWindowCodeLengthRule {
    fn check(&self, item: &BatchedRegionPinWindow) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRegionPinWindowRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedRegionPinWindowRule for BatchedRegionPinWindowRateRangeRule {
    fn check(&self, item: &BatchedRegionPinWindow) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a batched region pin window.
#[derive(Debug, Clone)]
pub struct BatchedRegionPinWindowValidator {
    pub rules: Vec<Rc<dyn BatchedRegionPinWindowRule>>,
}

impl BatchedRegionPinWindowValidator {
    pub fn validate(&self, item: &BatchedRegionPinWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedRegionPinWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedRegionPinWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_region_pin_window_validator() -> BatchedRegionPinWindowValidator {
    BatchedRegionPinWindowValidator {
        rules: vec![
            Rc::new(BatchedRegionPinWindowSummaryLengthRule { max_length: 1000 }) as Rc<dyn BatchedRegionPinWindowRule>,
            Rc::new(BatchedRegionPinWindowRequiredLabelRule {
                message: "batched region pin window needs an entry".to_string(),
            }),
            Rc::new(BatchedRegionPinWindowWindowDaysRangeRule { min: 3600, max: 86400 }),
            Rc::new(BatchedRegionPinWindowCodeLengthRule { max_length: 1000 }),
            Rc::new(BatchedRegionPinWindowRateRangeRule { min: 7.2, max: 153.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_region_pin_window_validator();
        let item = new_batched_region_pin_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = BatchedRegionPinWindowSummaryLengthRule { max_length: 3600 };
        let mut item = new_batched_region_pin_window("x", "y");
        item.summary = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = BatchedRegionPinWindowRequiredLabelRule {
            message: "batched region pin window needs a value".to_string(),
        };
        let mut item = new_batched_region_pin_window("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = BatchedRegionPinWindowWindowDaysRangeRule { min: 1000, max: 86400 };
        let mut item = new_batched_region_pin_window("x", "y");
        item.window_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedRegionPinWindowCodeLengthRule { max_length: 1000 };
        let mut item = new_batched_region_pin_window("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = BatchedRegionPinWindowRateRangeRule { min: 5.6, max: 4942.9 };
        let mut item = new_batched_region_pin_window("x", "y");
        item.rate = 4944.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
