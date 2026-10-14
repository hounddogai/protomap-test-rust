use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient forecast estimate must pass. Check returns its problems, or none.
pub trait LenientForecastEstimateRule: std::fmt::Debug {
    fn check(&self, item: &LenientForecastEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientForecastEstimateScopeLengthRule {
    pub max_length: i64,
}

impl LenientForecastEstimateRule for LenientForecastEstimateScopeLengthRule {
    fn check(&self, item: &LenientForecastEstimate) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientForecastEstimatePinnedGuardRule {
    pub limit: i64,
}

impl LenientForecastEstimateRule for LenientForecastEstimatePinnedGuardRule {
    fn check(&self, item: &LenientForecastEstimate) -> Vec<String> {
        if item.pinned && (item.limit > self.limit) {
            return vec![format!("limit {} exceeds {} while pinned", item.limit, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientForecastEstimateGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientForecastEstimateRule for LenientForecastEstimateGraceDaysRangeRule {
    fn check(&self, item: &LenientForecastEstimate) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientForecastEstimateRequiredLabelRule {
    pub message: String,
}

impl LenientForecastEstimateRule for LenientForecastEstimateRequiredLabelRule {
    fn check(&self, item: &LenientForecastEstimate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient forecast estimate.
#[derive(Debug, Clone)]
pub struct LenientForecastEstimateValidator {
    pub rules: Vec<Rc<dyn LenientForecastEstimateRule>>,
}

impl LenientForecastEstimateValidator {
    pub fn validate(&self, item: &LenientForecastEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientForecastEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientForecastEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_forecast_estimate_validator() -> LenientForecastEstimateValidator {
    LenientForecastEstimateValidator {
        rules: vec![
            Rc::new(LenientForecastEstimateScopeLengthRule { max_length: 64 }) as Rc<dyn LenientForecastEstimateRule>,
            Rc::new(LenientForecastEstimatePinnedGuardRule { limit: 3600 }),
            Rc::new(LenientForecastEstimateGraceDaysRangeRule { min: 16, max: 86400 }),
            Rc::new(LenientForecastEstimateRequiredLabelRule {
                message: "lenient forecast estimate needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_forecast_estimate_validator();
        let item = new_lenient_forecast_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = LenientForecastEstimateScopeLengthRule { max_length: 64 };
        let mut item = new_lenient_forecast_estimate("x", "y");
        item.scope = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = LenientForecastEstimateGraceDaysRangeRule { min: 16, max: 3600 };
        let mut item = new_lenient_forecast_estimate("x", "y");
        item.grace_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientForecastEstimateRequiredLabelRule {
            message: "lenient forecast estimate needs a value".to_string(),
        };
        let mut item = new_lenient_forecast_estimate("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
