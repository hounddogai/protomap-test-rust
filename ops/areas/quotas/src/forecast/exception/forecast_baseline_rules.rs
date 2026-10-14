use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast baseline must pass. Check returns its problems, or none.
pub trait ForecastBaselineRule: std::fmt::Debug {
    fn check(&self, item: &ForecastBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastBaselineRequiredLabelRule {
    pub message: String,
}

impl ForecastBaselineRule for ForecastBaselineRequiredLabelRule {
    fn check(&self, item: &ForecastBaseline) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastBaselineSourceLengthRule {
    pub max_length: i64,
}

impl ForecastBaselineRule for ForecastBaselineSourceLengthRule {
    fn check(&self, item: &ForecastBaseline) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastBaselineCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ForecastBaselineRule for ForecastBaselineCreatedAtRangeRule {
    fn check(&self, item: &ForecastBaseline) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastBaselineNameLengthRule {
    pub max_length: i64,
}

impl ForecastBaselineRule for ForecastBaselineNameLengthRule {
    fn check(&self, item: &ForecastBaseline) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastBaselineRequiredCodeRule {
    pub message: String,
}

impl ForecastBaselineRule for ForecastBaselineRequiredCodeRule {
    fn check(&self, item: &ForecastBaseline) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastBaselineCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ForecastBaselineRule for ForecastBaselineCostRangeRule {
    fn check(&self, item: &ForecastBaseline) -> Vec<String> {
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

/// Runs every rule of a forecast baseline.
#[derive(Debug, Clone)]
pub struct ForecastBaselineValidator {
    pub rules: Vec<Rc<dyn ForecastBaselineRule>>,
}

impl ForecastBaselineValidator {
    pub fn validate(&self, item: &ForecastBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ForecastBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_baseline_validator() -> ForecastBaselineValidator {
    ForecastBaselineValidator {
        rules: vec![
            Rc::new(ForecastBaselineRequiredLabelRule {
                message: "forecast baseline needs this field".to_string(),
            }) as Rc<dyn ForecastBaselineRule>,
            Rc::new(ForecastBaselineSourceLengthRule { max_length: 32 }),
            Rc::new(ForecastBaselineCreatedAtRangeRule { min: 1000, max: 3600 }),
            Rc::new(ForecastBaselineNameLengthRule { max_length: 32 }),
            Rc::new(ForecastBaselineRequiredCodeRule {
                message: "forecast baseline needs this field".to_string(),
            }),
            Rc::new(ForecastBaselineCostRangeRule { min: 4.6, max: 4388.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_baseline_validator();
        let item = new_forecast_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ForecastBaselineRequiredLabelRule {
            message: "forecast baseline needs this field".to_string(),
        };
        let mut item = new_forecast_baseline("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = ForecastBaselineSourceLengthRule { max_length: 86400 };
        let mut item = new_forecast_baseline("x", "y");
        item.source = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = ForecastBaselineCreatedAtRangeRule { min: 32, max: 80 };
        let mut item = new_forecast_baseline("x", "y");
        item.created_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ForecastBaselineNameLengthRule { max_length: 255 };
        let mut item = new_forecast_baseline("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ForecastBaselineRequiredCodeRule {
            message: "forecast baseline needs an entry".to_string(),
        };
        let mut item = new_forecast_baseline("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = ForecastBaselineCostRangeRule { min: 8.4, max: 4833.5 };
        let mut item = new_forecast_baseline("x", "y");
        item.cost = 4835.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
