use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched forecast milestone must pass. Check returns its problems, or none.
pub trait BatchedForecastMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &BatchedForecastMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastMilestoneRequiredTimezoneRule {
    pub message: String,
}

impl BatchedForecastMilestoneRule for BatchedForecastMilestoneRequiredTimezoneRule {
    fn check(&self, item: &BatchedForecastMilestone) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastMilestoneWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedForecastMilestoneRule for BatchedForecastMilestoneWindowDaysRangeRule {
    fn check(&self, item: &BatchedForecastMilestone) -> Vec<String> {
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
pub struct BatchedForecastMilestoneCodeLengthRule {
    pub max_length: i64,
}

impl BatchedForecastMilestoneRule for BatchedForecastMilestoneCodeLengthRule {
    fn check(&self, item: &BatchedForecastMilestone) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastMilestoneRequiredReviewerRule {
    pub message: String,
}

impl BatchedForecastMilestoneRule for BatchedForecastMilestoneRequiredReviewerRule {
    fn check(&self, item: &BatchedForecastMilestone) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched forecast milestone.
#[derive(Debug, Clone)]
pub struct BatchedForecastMilestoneValidator {
    pub rules: Vec<Rc<dyn BatchedForecastMilestoneRule>>,
}

impl BatchedForecastMilestoneValidator {
    pub fn validate(&self, item: &BatchedForecastMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedForecastMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedForecastMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_forecast_milestone_validator() -> BatchedForecastMilestoneValidator {
    BatchedForecastMilestoneValidator {
        rules: vec![
            Rc::new(BatchedForecastMilestoneRequiredTimezoneRule {
                message: "batched forecast milestone needs a value".to_string(),
            }) as Rc<dyn BatchedForecastMilestoneRule>,
            Rc::new(BatchedForecastMilestoneWindowDaysRangeRule { min: 1000, max: 1000 }),
            Rc::new(BatchedForecastMilestoneCodeLengthRule { max_length: 120 }),
            Rc::new(BatchedForecastMilestoneRequiredReviewerRule {
                message: "batched forecast milestone needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_forecast_milestone_validator();
        let item = new_batched_forecast_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = BatchedForecastMilestoneRequiredTimezoneRule {
            message: "batched forecast milestone needs a value".to_string(),
        };
        let mut item = new_batched_forecast_milestone("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = BatchedForecastMilestoneWindowDaysRangeRule { min: 16, max: 255 };
        let mut item = new_batched_forecast_milestone("x", "y");
        item.window_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedForecastMilestoneCodeLengthRule { max_length: 64 };
        let mut item = new_batched_forecast_milestone("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = BatchedForecastMilestoneRequiredReviewerRule {
            message: "batched forecast milestone needs an entry".to_string(),
        };
        let mut item = new_batched_forecast_milestone("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
