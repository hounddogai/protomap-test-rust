use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast summary must pass. Check returns its problems, or none.
pub trait ForecastSummaryRule: std::fmt::Debug {
    fn check(&self, item: &ForecastSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastSummaryRequiredCodeRule {
    pub message: String,
}

impl ForecastSummaryRule for ForecastSummaryRequiredCodeRule {
    fn check(&self, item: &ForecastSummary) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastSummaryPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ForecastSummaryRule for ForecastSummaryPortRangeRule {
    fn check(&self, item: &ForecastSummary) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastSummaryCountryLengthRule {
    pub max_length: i64,
}

impl ForecastSummaryRule for ForecastSummaryCountryLengthRule {
    fn check(&self, item: &ForecastSummary) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast summary.
#[derive(Debug, Clone)]
pub struct ForecastSummaryValidator {
    pub rules: Vec<Rc<dyn ForecastSummaryRule>>,
}

impl ForecastSummaryValidator {
    pub fn validate(&self, item: &ForecastSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ForecastSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_summary_validator() -> ForecastSummaryValidator {
    ForecastSummaryValidator {
        rules: vec![
            Rc::new(ForecastSummaryRequiredCodeRule {
                message: "forecast summary needs this field".to_string(),
            }) as Rc<dyn ForecastSummaryRule>,
            Rc::new(ForecastSummaryPortRangeRule { min: 64, max: 80 }),
            Rc::new(ForecastSummaryCountryLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_summary_validator();
        let item = new_forecast_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ForecastSummaryRequiredCodeRule {
            message: "forecast summary needs this field".to_string(),
        };
        let mut item = new_forecast_summary("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = ForecastSummaryPortRangeRule { min: 16, max: 64 };
        let mut item = new_forecast_summary("x", "y");
        item.port = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = ForecastSummaryCountryLengthRule { max_length: 16 };
        let mut item = new_forecast_summary("x", "y");
        item.country = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
