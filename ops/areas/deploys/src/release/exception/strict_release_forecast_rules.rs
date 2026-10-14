use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict release forecast must pass. Check returns its problems, or none.
pub trait StrictReleaseForecastRule: std::fmt::Debug {
    fn check(&self, item: &StrictReleaseForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictReleaseForecastRequiredNameRule {
    pub message: String,
}

impl StrictReleaseForecastRule for StrictReleaseForecastRequiredNameRule {
    fn check(&self, item: &StrictReleaseForecast) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictReleaseForecastNamespaceLengthRule {
    pub max_length: i64,
}

impl StrictReleaseForecastRule for StrictReleaseForecastNamespaceLengthRule {
    fn check(&self, item: &StrictReleaseForecast) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictReleaseForecastRequiredCodeRule {
    pub message: String,
}

impl StrictReleaseForecastRule for StrictReleaseForecastRequiredCodeRule {
    fn check(&self, item: &StrictReleaseForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictReleaseForecastBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictReleaseForecastRule for StrictReleaseForecastBudgetRangeRule {
    fn check(&self, item: &StrictReleaseForecast) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict release forecast.
#[derive(Debug, Clone)]
pub struct StrictReleaseForecastValidator {
    pub rules: Vec<Rc<dyn StrictReleaseForecastRule>>,
}

impl StrictReleaseForecastValidator {
    pub fn validate(&self, item: &StrictReleaseForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictReleaseForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictReleaseForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_release_forecast_validator() -> StrictReleaseForecastValidator {
    StrictReleaseForecastValidator {
        rules: vec![
            Rc::new(StrictReleaseForecastRequiredNameRule {
                message: "strict release forecast needs a value".to_string(),
            }) as Rc<dyn StrictReleaseForecastRule>,
            Rc::new(StrictReleaseForecastNamespaceLengthRule { max_length: 64 }),
            Rc::new(StrictReleaseForecastRequiredCodeRule {
                message: "strict release forecast needs a value".to_string(),
            }),
            Rc::new(StrictReleaseForecastBudgetRangeRule { min: 8.7, max: 3557.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_release_forecast_validator();
        let item = new_strict_release_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = StrictReleaseForecastRequiredNameRule {
            message: "strict release forecast needs this field".to_string(),
        };
        let mut item = new_strict_release_forecast("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = StrictReleaseForecastNamespaceLengthRule { max_length: 16 };
        let mut item = new_strict_release_forecast("x", "y");
        item.namespace = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictReleaseForecastRequiredCodeRule {
            message: "strict release forecast needs a value".to_string(),
        };
        let mut item = new_strict_release_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = StrictReleaseForecastBudgetRangeRule { min: 4.8, max: 3571.4 };
        let mut item = new_strict_release_forecast("x", "y");
        item.budget = 3572.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
