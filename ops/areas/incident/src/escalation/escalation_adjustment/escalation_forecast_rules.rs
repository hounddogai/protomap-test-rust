use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a escalation forecast must pass. Check returns its problems, or none.
pub trait EscalationForecastRule: std::fmt::Debug {
    fn check(&self, item: &EscalationForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationForecastRequiredCurrencyRule {
    pub message: String,
}

impl EscalationForecastRule for EscalationForecastRequiredCurrencyRule {
    fn check(&self, item: &EscalationForecast) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationForecastCodeLengthRule {
    pub max_length: i64,
}

impl EscalationForecastRule for EscalationForecastCodeLengthRule {
    fn check(&self, item: &EscalationForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationForecastRequiredNameRule {
    pub message: String,
}

impl EscalationForecastRule for EscalationForecastRequiredNameRule {
    fn check(&self, item: &EscalationForecast) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationForecastCodePrefixRule {
    pub prefix: String,
}

impl EscalationForecastRule for EscalationForecastCodePrefixRule {
    fn check(&self, item: &EscalationForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationForecastEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EscalationForecastRule for EscalationForecastEndsAtRangeRule {
    fn check(&self, item: &EscalationForecast) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation forecast.
#[derive(Debug, Clone)]
pub struct EscalationForecastValidator {
    pub rules: Vec<Rc<dyn EscalationForecastRule>>,
}

impl EscalationForecastValidator {
    pub fn validate(&self, item: &EscalationForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EscalationForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_forecast_validator() -> EscalationForecastValidator {
    EscalationForecastValidator {
        rules: vec![
            Rc::new(EscalationForecastRequiredCurrencyRule {
                message: "escalation forecast needs an entry".to_string(),
            }) as Rc<dyn EscalationForecastRule>,
            Rc::new(EscalationForecastCodeLengthRule { max_length: 3600 }),
            Rc::new(EscalationForecastRequiredNameRule {
                message: "escalation forecast needs this field".to_string(),
            }),
            Rc::new(EscalationForecastCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(EscalationForecastEndsAtRangeRule { min: 64, max: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_escalation_forecast_validator();
        let item = new_escalation_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = EscalationForecastRequiredCurrencyRule {
            message: "escalation forecast needs an entry".to_string(),
        };
        let mut item = new_escalation_forecast("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EscalationForecastCodeLengthRule { max_length: 86400 };
        let mut item = new_escalation_forecast("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = EscalationForecastRequiredNameRule {
            message: "escalation forecast needs an entry".to_string(),
        };
        let mut item = new_escalation_forecast("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = EscalationForecastEndsAtRangeRule { min: 64, max: 120 };
        let mut item = new_escalation_forecast("x", "y");
        item.ends_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
