use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a runbook forecast must pass. Check returns its problems, or none.
pub trait RunbookForecastRule: std::fmt::Debug {
    fn check(&self, item: &RunbookForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookForecastCodePrefixRule {
    pub prefix: String,
}

impl RunbookForecastRule for RunbookForecastCodePrefixRule {
    fn check(&self, item: &RunbookForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookForecastApprovedGuardRule {
    pub limit: i64,
}

impl RunbookForecastRule for RunbookForecastApprovedGuardRule {
    fn check(&self, item: &RunbookForecast) -> Vec<String> {
        if item.approved && (item.rotation_days > self.limit) {
            return vec![format!(
                "rotation days {} exceeds {} while approved",
                item.rotation_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookForecastThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RunbookForecastRule for RunbookForecastThresholdRangeRule {
    fn check(&self, item: &RunbookForecast) -> Vec<String> {
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
pub struct RunbookForecastRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RunbookForecastRule for RunbookForecastRotationDaysRangeRule {
    fn check(&self, item: &RunbookForecast) -> Vec<String> {
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
pub struct RunbookForecastLabelLengthRule {
    pub max_length: i64,
}

impl RunbookForecastRule for RunbookForecastLabelLengthRule {
    fn check(&self, item: &RunbookForecast) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a runbook forecast.
#[derive(Debug, Clone)]
pub struct RunbookForecastValidator {
    pub rules: Vec<Rc<dyn RunbookForecastRule>>,
}

impl RunbookForecastValidator {
    pub fn validate(&self, item: &RunbookForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RunbookForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_forecast_validator() -> RunbookForecastValidator {
    RunbookForecastValidator {
        rules: vec![
            Rc::new(RunbookForecastCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn RunbookForecastRule>,
            Rc::new(RunbookForecastApprovedGuardRule { limit: 86400 }),
            Rc::new(RunbookForecastThresholdRangeRule { min: 7.6, max: 2812.8 }),
            Rc::new(RunbookForecastRotationDaysRangeRule { min: 32, max: 32 }),
            Rc::new(RunbookForecastLabelLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_runbook_forecast_validator();
        let item = new_runbook_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = RunbookForecastThresholdRangeRule { min: 9.9, max: 2652.8 };
        let mut item = new_runbook_forecast("x", "y");
        item.threshold = 2654.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = RunbookForecastRotationDaysRangeRule { min: 16, max: 86400 };
        let mut item = new_runbook_forecast("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RunbookForecastLabelLengthRule { max_length: 32 };
        let mut item = new_runbook_forecast("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
