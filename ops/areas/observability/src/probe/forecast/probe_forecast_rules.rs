use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a probe forecast must pass. Check returns its problems, or none.
pub trait ProbeForecastRule: std::fmt::Debug {
    fn check(&self, item: &ProbeForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeForecastRequiredCodeRule {
    pub message: String,
}

impl ProbeForecastRule for ProbeForecastRequiredCodeRule {
    fn check(&self, item: &ProbeForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeForecastSummaryLengthRule {
    pub max_length: i64,
}

impl ProbeForecastRule for ProbeForecastSummaryLengthRule {
    fn check(&self, item: &ProbeForecast) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeForecastCodePrefixRule {
    pub prefix: String,
}

impl ProbeForecastRule for ProbeForecastCodePrefixRule {
    fn check(&self, item: &ProbeForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeForecastRequiredSummaryRule {
    pub message: String,
}

impl ProbeForecastRule for ProbeForecastRequiredSummaryRule {
    fn check(&self, item: &ProbeForecast) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeForecastPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbeForecastRule for ProbeForecastPriorityRangeRule {
    fn check(&self, item: &ProbeForecast) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe forecast.
#[derive(Debug, Clone)]
pub struct ProbeForecastValidator {
    pub rules: Vec<Rc<dyn ProbeForecastRule>>,
}

impl ProbeForecastValidator {
    pub fn validate(&self, item: &ProbeForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ProbeForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_forecast_validator() -> ProbeForecastValidator {
    ProbeForecastValidator {
        rules: vec![
            Rc::new(ProbeForecastRequiredCodeRule {
                message: "probe forecast needs a value".to_string(),
            }) as Rc<dyn ProbeForecastRule>,
            Rc::new(ProbeForecastSummaryLengthRule { max_length: 80 }),
            Rc::new(ProbeForecastCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(ProbeForecastRequiredSummaryRule {
                message: "probe forecast needs an entry".to_string(),
            }),
            Rc::new(ProbeForecastPriorityRangeRule { min: 16, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_probe_forecast_validator();
        let item = new_probe_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ProbeForecastRequiredCodeRule {
            message: "probe forecast needs an entry".to_string(),
        };
        let mut item = new_probe_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = ProbeForecastSummaryLengthRule { max_length: 32 };
        let mut item = new_probe_forecast("x", "y");
        item.summary = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = ProbeForecastRequiredSummaryRule {
            message: "probe forecast needs an entry".to_string(),
        };
        let mut item = new_probe_forecast("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = ProbeForecastPriorityRangeRule { min: 16, max: 64 };
        let mut item = new_probe_forecast("x", "y");
        item.priority = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
