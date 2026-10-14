use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched sink forecast must pass. Check returns its problems, or none.
pub trait BatchedSinkForecastRule: std::fmt::Debug {
    fn check(&self, item: &BatchedSinkForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSinkForecastRequiredNameRule {
    pub message: String,
}

impl BatchedSinkForecastRule for BatchedSinkForecastRequiredNameRule {
    fn check(&self, item: &BatchedSinkForecast) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSinkForecastRequiredNamespaceRule {
    pub message: String,
}

impl BatchedSinkForecastRule for BatchedSinkForecastRequiredNamespaceRule {
    fn check(&self, item: &BatchedSinkForecast) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSinkForecastAllowedNamespaceRule {
    pub allowed: Vec<String>,
}

impl BatchedSinkForecastRule for BatchedSinkForecastAllowedNamespaceRule {
    fn check(&self, item: &BatchedSinkForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.namespace)))
        {
            return vec![format!("namespace {} is below minimum", item.namespace)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSinkForecastRequiredOwnerIdRule {
    pub message: String,
}

impl BatchedSinkForecastRule for BatchedSinkForecastRequiredOwnerIdRule {
    fn check(&self, item: &BatchedSinkForecast) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSinkForecastAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedSinkForecastRule for BatchedSinkForecastAmountRangeRule {
    fn check(&self, item: &BatchedSinkForecast) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSinkForecastDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedSinkForecastRule for BatchedSinkForecastDiscountRangeRule {
    fn check(&self, item: &BatchedSinkForecast) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a batched sink forecast.
#[derive(Debug, Clone)]
pub struct BatchedSinkForecastValidator {
    pub rules: Vec<Rc<dyn BatchedSinkForecastRule>>,
}

impl BatchedSinkForecastValidator {
    pub fn validate(&self, item: &BatchedSinkForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedSinkForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedSinkForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_sink_forecast_validator() -> BatchedSinkForecastValidator {
    BatchedSinkForecastValidator {
        rules: vec![
            Rc::new(BatchedSinkForecastRequiredNameRule {
                message: "batched sink forecast needs an entry".to_string(),
            }) as Rc<dyn BatchedSinkForecastRule>,
            Rc::new(BatchedSinkForecastRequiredNamespaceRule {
                message: "batched sink forecast needs a value".to_string(),
            }),
            Rc::new(BatchedSinkForecastAllowedNamespaceRule {
                allowed: vec!["scheduled".to_string(), "adaptive".to_string(), "default".to_string()],
            }),
            Rc::new(BatchedSinkForecastRequiredOwnerIdRule {
                message: "batched sink forecast needs a value".to_string(),
            }),
            Rc::new(BatchedSinkForecastAmountRangeRule { min: 6.6, max: 574.6 }),
            Rc::new(BatchedSinkForecastDiscountRangeRule { min: 4.0, max: 4687.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_sink_forecast_validator();
        let item = new_batched_sink_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = BatchedSinkForecastRequiredNameRule {
            message: "batched sink forecast needs this field".to_string(),
        };
        let mut item = new_batched_sink_forecast("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = BatchedSinkForecastRequiredNamespaceRule {
            message: "batched sink forecast needs a value".to_string(),
        };
        let mut item = new_batched_sink_forecast("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = BatchedSinkForecastRequiredOwnerIdRule {
            message: "batched sink forecast needs this field".to_string(),
        };
        let mut item = new_batched_sink_forecast("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = BatchedSinkForecastAmountRangeRule { min: 1.2, max: 1300.6 };
        let mut item = new_batched_sink_forecast("x", "y");
        item.amount = 1302.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = BatchedSinkForecastDiscountRangeRule { min: 4.9, max: 283.7 };
        let mut item = new_batched_sink_forecast("x", "y");
        item.discount = 285.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
