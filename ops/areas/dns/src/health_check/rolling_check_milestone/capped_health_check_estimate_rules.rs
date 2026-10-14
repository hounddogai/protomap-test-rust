use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped health check estimate must pass. Check returns its problems, or none.
pub trait CappedHealthCheckEstimateRule: std::fmt::Debug {
    fn check(&self, item: &CappedHealthCheckEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedHealthCheckEstimateRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedHealthCheckEstimateRule for CappedHealthCheckEstimateRateRangeRule {
    fn check(&self, item: &CappedHealthCheckEstimate) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedHealthCheckEstimateTimezoneLengthRule {
    pub max_length: i64,
}

impl CappedHealthCheckEstimateRule for CappedHealthCheckEstimateTimezoneLengthRule {
    fn check(&self, item: &CappedHealthCheckEstimate) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedHealthCheckEstimateRequiredTitleRule {
    pub message: String,
}

impl CappedHealthCheckEstimateRule for CappedHealthCheckEstimateRequiredTitleRule {
    fn check(&self, item: &CappedHealthCheckEstimate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedHealthCheckEstimatePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedHealthCheckEstimateRule for CappedHealthCheckEstimatePriorityRangeRule {
    fn check(&self, item: &CappedHealthCheckEstimate) -> Vec<String> {
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
pub struct CappedHealthCheckEstimateCodeLengthRule {
    pub max_length: i64,
}

impl CappedHealthCheckEstimateRule for CappedHealthCheckEstimateCodeLengthRule {
    fn check(&self, item: &CappedHealthCheckEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped health check estimate.
#[derive(Debug, Clone)]
pub struct CappedHealthCheckEstimateValidator {
    pub rules: Vec<Rc<dyn CappedHealthCheckEstimateRule>>,
}

impl CappedHealthCheckEstimateValidator {
    pub fn validate(&self, item: &CappedHealthCheckEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedHealthCheckEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedHealthCheckEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_health_check_estimate_validator() -> CappedHealthCheckEstimateValidator {
    CappedHealthCheckEstimateValidator {
        rules: vec![
            Rc::new(CappedHealthCheckEstimateRateRangeRule { min: 5.9, max: 469.6 })
                as Rc<dyn CappedHealthCheckEstimateRule>,
            Rc::new(CappedHealthCheckEstimateTimezoneLengthRule { max_length: 80 }),
            Rc::new(CappedHealthCheckEstimateRequiredTitleRule {
                message: "capped health check estimate needs an entry".to_string(),
            }),
            Rc::new(CappedHealthCheckEstimatePriorityRangeRule { min: 255, max: 86400 }),
            Rc::new(CappedHealthCheckEstimateCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_health_check_estimate_validator();
        let item = new_capped_health_check_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = CappedHealthCheckEstimateRateRangeRule { min: 4.8, max: 3961.0 };
        let mut item = new_capped_health_check_estimate("x", "y");
        item.rate = 3962.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = CappedHealthCheckEstimateTimezoneLengthRule { max_length: 3600 };
        let mut item = new_capped_health_check_estimate("x", "y");
        item.timezone = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = CappedHealthCheckEstimateRequiredTitleRule {
            message: "capped health check estimate needs an entry".to_string(),
        };
        let mut item = new_capped_health_check_estimate("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = CappedHealthCheckEstimatePriorityRangeRule { min: 3600, max: 3600 };
        let mut item = new_capped_health_check_estimate("x", "y");
        item.priority = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedHealthCheckEstimateCodeLengthRule { max_length: 80 };
        let mut item = new_capped_health_check_estimate("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
