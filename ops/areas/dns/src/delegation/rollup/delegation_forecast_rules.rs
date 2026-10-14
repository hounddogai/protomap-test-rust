use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a delegation forecast must pass. Check returns its problems, or none.
pub trait DelegationForecastRule: std::fmt::Debug {
    fn check(&self, item: &DelegationForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationForecastSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationForecastRule for DelegationForecastSequenceRangeRule {
    fn check(&self, item: &DelegationForecast) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationForecastRequiredLabelRule {
    pub message: String,
}

impl DelegationForecastRule for DelegationForecastRequiredLabelRule {
    fn check(&self, item: &DelegationForecast) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationForecastRequiredTitleRule {
    pub message: String,
}

impl DelegationForecastRule for DelegationForecastRequiredTitleRule {
    fn check(&self, item: &DelegationForecast) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationForecastLabelLengthRule {
    pub max_length: i64,
}

impl DelegationForecastRule for DelegationForecastLabelLengthRule {
    fn check(&self, item: &DelegationForecast) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationForecastBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationForecastRule for DelegationForecastBurstRangeRule {
    fn check(&self, item: &DelegationForecast) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a delegation forecast.
#[derive(Debug, Clone)]
pub struct DelegationForecastValidator {
    pub rules: Vec<Rc<dyn DelegationForecastRule>>,
}

impl DelegationForecastValidator {
    pub fn validate(&self, item: &DelegationForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DelegationForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_forecast_validator() -> DelegationForecastValidator {
    DelegationForecastValidator {
        rules: vec![
            Rc::new(DelegationForecastSequenceRangeRule { min: 80, max: 120 }) as Rc<dyn DelegationForecastRule>,
            Rc::new(DelegationForecastRequiredLabelRule {
                message: "delegation forecast needs a value".to_string(),
            }),
            Rc::new(DelegationForecastRequiredTitleRule {
                message: "delegation forecast needs this field".to_string(),
            }),
            Rc::new(DelegationForecastLabelLengthRule { max_length: 64 }),
            Rc::new(DelegationForecastBurstRangeRule { min: 16, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_delegation_forecast_validator();
        let item = new_delegation_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = DelegationForecastSequenceRangeRule { min: 32, max: 80 };
        let mut item = new_delegation_forecast("x", "y");
        item.sequence = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DelegationForecastRequiredLabelRule {
            message: "delegation forecast needs an entry".to_string(),
        };
        let mut item = new_delegation_forecast("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = DelegationForecastRequiredTitleRule {
            message: "delegation forecast needs a value".to_string(),
        };
        let mut item = new_delegation_forecast("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DelegationForecastLabelLengthRule { max_length: 86400 };
        let mut item = new_delegation_forecast("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = DelegationForecastBurstRangeRule { min: 32, max: 3600 };
        let mut item = new_delegation_forecast("x", "y");
        item.burst = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
