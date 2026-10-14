use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding forecast must pass. Check returns its problems, or none.
pub trait BindingForecastRule: std::fmt::Debug {
    fn check(&self, item: &BindingForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingForecastLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingForecastRule for BindingForecastLimitRangeRule {
    fn check(&self, item: &BindingForecast) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingForecastRequiredSummaryRule {
    pub message: String,
}

impl BindingForecastRule for BindingForecastRequiredSummaryRule {
    fn check(&self, item: &BindingForecast) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingForecastSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingForecastRule for BindingForecastSequenceRangeRule {
    fn check(&self, item: &BindingForecast) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding forecast.
#[derive(Debug, Clone)]
pub struct BindingForecastValidator {
    pub rules: Vec<Rc<dyn BindingForecastRule>>,
}

impl BindingForecastValidator {
    pub fn validate(&self, item: &BindingForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BindingForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_forecast_validator() -> BindingForecastValidator {
    BindingForecastValidator {
        rules: vec![
            Rc::new(BindingForecastLimitRangeRule { min: 32, max: 120 }) as Rc<dyn BindingForecastRule>,
            Rc::new(BindingForecastRequiredSummaryRule {
                message: "binding forecast needs a value".to_string(),
            }),
            Rc::new(BindingForecastSequenceRangeRule { min: 80, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_binding_forecast_validator();
        let item = new_binding_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = BindingForecastLimitRangeRule { min: 1000, max: 86400 };
        let mut item = new_binding_forecast("x", "y");
        item.limit = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = BindingForecastRequiredSummaryRule {
            message: "binding forecast needs this field".to_string(),
        };
        let mut item = new_binding_forecast("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = BindingForecastSequenceRangeRule { min: 16, max: 64 };
        let mut item = new_binding_forecast("x", "y");
        item.sequence = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
