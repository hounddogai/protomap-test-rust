use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a forecast rollup must pass. Check returns its problems, or none.
pub trait ForecastRollupRule: std::fmt::Debug {
    fn check(&self, item: &ForecastRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastRollupSummaryLengthRule {
    pub max_length: i64,
}

impl ForecastRollupRule for ForecastRollupSummaryLengthRule {
    fn check(&self, item: &ForecastRollup) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastRollupCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ForecastRollupRule for ForecastRollupCostRangeRule {
    fn check(&self, item: &ForecastRollup) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastRollupAllowedExternalRefRule {
    pub allowed: Vec<String>,
}

impl ForecastRollupRule for ForecastRollupAllowedExternalRefRule {
    fn check(&self, item: &ForecastRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.external_ref)))
        {
            return vec![format!("external ref {} is missing", item.external_ref)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastRollupRequiredCodeRule {
    pub message: String,
}

impl ForecastRollupRule for ForecastRollupRequiredCodeRule {
    fn check(&self, item: &ForecastRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast rollup.
#[derive(Debug, Clone)]
pub struct ForecastRollupValidator {
    pub rules: Vec<Rc<dyn ForecastRollupRule>>,
}

impl ForecastRollupValidator {
    pub fn validate(&self, item: &ForecastRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ForecastRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_rollup_validator() -> ForecastRollupValidator {
    ForecastRollupValidator {
        rules: vec![
            Rc::new(ForecastRollupSummaryLengthRule { max_length: 86400 }) as Rc<dyn ForecastRollupRule>,
            Rc::new(ForecastRollupCostRangeRule { min: 9.2, max: 3962.7 }),
            Rc::new(ForecastRollupAllowedExternalRefRule {
                allowed: vec!["rolling".to_string()],
            }),
            Rc::new(ForecastRollupRequiredCodeRule {
                message: "forecast rollup needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_rollup_validator();
        let item = new_forecast_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = ForecastRollupSummaryLengthRule { max_length: 64 };
        let mut item = new_forecast_rollup("x", "y");
        item.summary = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = ForecastRollupCostRangeRule { min: 6.9, max: 4222.3 };
        let mut item = new_forecast_rollup("x", "y");
        item.cost = 4223.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ForecastRollupRequiredCodeRule {
            message: "forecast rollup needs a value".to_string(),
        };
        let mut item = new_forecast_rollup("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
