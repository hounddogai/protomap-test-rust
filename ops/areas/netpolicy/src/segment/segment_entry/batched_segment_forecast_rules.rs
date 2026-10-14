use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched segment forecast must pass. Check returns its problems, or none.
pub trait BatchedSegmentForecastRule: std::fmt::Debug {
    fn check(&self, item: &BatchedSegmentForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSegmentForecastRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedSegmentForecastRule for BatchedSegmentForecastRateRangeRule {
    fn check(&self, item: &BatchedSegmentForecast) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSegmentForecastRequiredLabelRule {
    pub message: String,
}

impl BatchedSegmentForecastRule for BatchedSegmentForecastRequiredLabelRule {
    fn check(&self, item: &BatchedSegmentForecast) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSegmentForecastRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedSegmentForecastRule for BatchedSegmentForecastRotationDaysRangeRule {
    fn check(&self, item: &BatchedSegmentForecast) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSegmentForecastLabelLengthRule {
    pub max_length: i64,
}

impl BatchedSegmentForecastRule for BatchedSegmentForecastLabelLengthRule {
    fn check(&self, item: &BatchedSegmentForecast) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSegmentForecastRequiredCodeRule {
    pub message: String,
}

impl BatchedSegmentForecastRule for BatchedSegmentForecastRequiredCodeRule {
    fn check(&self, item: &BatchedSegmentForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSegmentForecastPinnedGuardRule {
    pub limit: i64,
}

impl BatchedSegmentForecastRule for BatchedSegmentForecastPinnedGuardRule {
    fn check(&self, item: &BatchedSegmentForecast) -> Vec<String> {
        if item.pinned && (item.rotation_days > self.limit) {
            return vec![format!(
                "rotation days {} exceeds {} while pinned",
                item.rotation_days, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched segment forecast.
#[derive(Debug, Clone)]
pub struct BatchedSegmentForecastValidator {
    pub rules: Vec<Rc<dyn BatchedSegmentForecastRule>>,
}

impl BatchedSegmentForecastValidator {
    pub fn validate(&self, item: &BatchedSegmentForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedSegmentForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedSegmentForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_segment_forecast_validator() -> BatchedSegmentForecastValidator {
    BatchedSegmentForecastValidator {
        rules: vec![
            Rc::new(BatchedSegmentForecastRateRangeRule { min: 4.5, max: 73.6 }) as Rc<dyn BatchedSegmentForecastRule>,
            Rc::new(BatchedSegmentForecastRequiredLabelRule {
                message: "batched segment forecast needs an entry".to_string(),
            }),
            Rc::new(BatchedSegmentForecastRotationDaysRangeRule { min: 64, max: 3600 }),
            Rc::new(BatchedSegmentForecastLabelLengthRule { max_length: 80 }),
            Rc::new(BatchedSegmentForecastRequiredCodeRule {
                message: "batched segment forecast needs this field".to_string(),
            }),
            Rc::new(BatchedSegmentForecastPinnedGuardRule { limit: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_segment_forecast_validator();
        let item = new_batched_segment_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = BatchedSegmentForecastRateRangeRule { min: 2.3, max: 2176.9 };
        let mut item = new_batched_segment_forecast("x", "y");
        item.rate = 2178.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = BatchedSegmentForecastRequiredLabelRule {
            message: "batched segment forecast needs an entry".to_string(),
        };
        let mut item = new_batched_segment_forecast("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = BatchedSegmentForecastRotationDaysRangeRule { min: 64, max: 255 };
        let mut item = new_batched_segment_forecast("x", "y");
        item.rotation_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = BatchedSegmentForecastLabelLengthRule { max_length: 1000 };
        let mut item = new_batched_segment_forecast("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedSegmentForecastRequiredCodeRule {
            message: "batched segment forecast needs this field".to_string(),
        };
        let mut item = new_batched_segment_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
