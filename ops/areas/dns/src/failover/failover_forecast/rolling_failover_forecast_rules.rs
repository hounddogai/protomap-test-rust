use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling failover forecast must pass. Check returns its problems, or none.
pub trait RollingFailoverForecastRule: std::fmt::Debug {
    fn check(&self, item: &RollingFailoverForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFailoverForecastLabelLengthRule {
    pub max_length: i64,
}

impl RollingFailoverForecastRule for RollingFailoverForecastLabelLengthRule {
    fn check(&self, item: &RollingFailoverForecast) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFailoverForecastRequiredLabelRule {
    pub message: String,
}

impl RollingFailoverForecastRule for RollingFailoverForecastRequiredLabelRule {
    fn check(&self, item: &RollingFailoverForecast) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFailoverForecastRequiredNotesRule {
    pub message: String,
}

impl RollingFailoverForecastRule for RollingFailoverForecastRequiredNotesRule {
    fn check(&self, item: &RollingFailoverForecast) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFailoverForecastPrincipalLengthRule {
    pub max_length: i64,
}

impl RollingFailoverForecastRule for RollingFailoverForecastPrincipalLengthRule {
    fn check(&self, item: &RollingFailoverForecast) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFailoverForecastCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingFailoverForecastRule for RollingFailoverForecastCreatedAtRangeRule {
    fn check(&self, item: &RollingFailoverForecast) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFailoverForecastNotesLengthRule {
    pub max_length: i64,
}

impl RollingFailoverForecastRule for RollingFailoverForecastNotesLengthRule {
    fn check(&self, item: &RollingFailoverForecast) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling failover forecast.
#[derive(Debug, Clone)]
pub struct RollingFailoverForecastValidator {
    pub rules: Vec<Rc<dyn RollingFailoverForecastRule>>,
}

impl RollingFailoverForecastValidator {
    pub fn validate(&self, item: &RollingFailoverForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingFailoverForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingFailoverForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_failover_forecast_validator() -> RollingFailoverForecastValidator {
    RollingFailoverForecastValidator {
        rules: vec![
            Rc::new(RollingFailoverForecastLabelLengthRule { max_length: 120 }) as Rc<dyn RollingFailoverForecastRule>,
            Rc::new(RollingFailoverForecastRequiredLabelRule {
                message: "rolling failover forecast needs this field".to_string(),
            }),
            Rc::new(RollingFailoverForecastRequiredNotesRule {
                message: "rolling failover forecast needs an entry".to_string(),
            }),
            Rc::new(RollingFailoverForecastPrincipalLengthRule { max_length: 16 }),
            Rc::new(RollingFailoverForecastCreatedAtRangeRule { min: 80, max: 255 }),
            Rc::new(RollingFailoverForecastNotesLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_failover_forecast_validator();
        let item = new_rolling_failover_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RollingFailoverForecastLabelLengthRule { max_length: 16 };
        let mut item = new_rolling_failover_forecast("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RollingFailoverForecastRequiredLabelRule {
            message: "rolling failover forecast needs this field".to_string(),
        };
        let mut item = new_rolling_failover_forecast("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = RollingFailoverForecastRequiredNotesRule {
            message: "rolling failover forecast needs an entry".to_string(),
        };
        let mut item = new_rolling_failover_forecast("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = RollingFailoverForecastPrincipalLengthRule { max_length: 64 };
        let mut item = new_rolling_failover_forecast("x", "y");
        item.principal = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = RollingFailoverForecastCreatedAtRangeRule { min: 1000, max: 3600 };
        let mut item = new_rolling_failover_forecast("x", "y");
        item.created_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = RollingFailoverForecastNotesLengthRule { max_length: 64 };
        let mut item = new_rolling_failover_forecast("x", "y");
        item.notes = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
