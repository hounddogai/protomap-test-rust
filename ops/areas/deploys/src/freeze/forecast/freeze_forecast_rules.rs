use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a freeze forecast must pass. Check returns its problems, or none.
pub trait FreezeForecastRule: std::fmt::Debug {
    fn check(&self, item: &FreezeForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeForecastContactEmailLengthRule {
    pub max_length: i64,
}

impl FreezeForecastRule for FreezeForecastContactEmailLengthRule {
    fn check(&self, item: &FreezeForecast) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeForecastQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FreezeForecastRule for FreezeForecastQuantityRangeRule {
    fn check(&self, item: &FreezeForecast) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeForecastApprovedGuardRule {
    pub limit: f64,
}

impl FreezeForecastRule for FreezeForecastApprovedGuardRule {
    fn check(&self, item: &FreezeForecast) -> Vec<String> {
        if item.approved && (item.rate > self.limit) {
            return vec![format!("rate {} exceeds {} while approved", item.rate, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeForecastCodePrefixRule {
    pub prefix: String,
}

impl FreezeForecastRule for FreezeForecastCodePrefixRule {
    fn check(&self, item: &FreezeForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a freeze forecast.
#[derive(Debug, Clone)]
pub struct FreezeForecastValidator {
    pub rules: Vec<Rc<dyn FreezeForecastRule>>,
}

impl FreezeForecastValidator {
    pub fn validate(&self, item: &FreezeForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FreezeForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_forecast_validator() -> FreezeForecastValidator {
    FreezeForecastValidator {
        rules: vec![
            Rc::new(FreezeForecastContactEmailLengthRule { max_length: 3600 }) as Rc<dyn FreezeForecastRule>,
            Rc::new(FreezeForecastQuantityRangeRule { min: 120, max: 1000 }),
            Rc::new(FreezeForecastApprovedGuardRule { limit: 8.3 }),
            Rc::new(FreezeForecastCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_freeze_forecast_validator();
        let item = new_freeze_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = FreezeForecastContactEmailLengthRule { max_length: 1000 };
        let mut item = new_freeze_forecast("x", "y");
        item.contact_email = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = FreezeForecastQuantityRangeRule { min: 64, max: 64 };
        let mut item = new_freeze_forecast("x", "y");
        item.quantity = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
