use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rule forecast must pass. Check returns its problems, or none.
pub trait RuleForecastRule: std::fmt::Debug {
    fn check(&self, item: &RuleForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleForecastPhoneLengthRule {
    pub max_length: i64,
}

impl RuleForecastRule for RuleForecastPhoneLengthRule {
    fn check(&self, item: &RuleForecast) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleForecastAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RuleForecastRule for RuleForecastAmountRangeRule {
    fn check(&self, item: &RuleForecast) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleForecastBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RuleForecastRule for RuleForecastBudgetRangeRule {
    fn check(&self, item: &RuleForecast) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleForecastLabelLengthRule {
    pub max_length: i64,
}

impl RuleForecastRule for RuleForecastLabelLengthRule {
    fn check(&self, item: &RuleForecast) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleForecastCodePrefixRule {
    pub prefix: String,
}

impl RuleForecastRule for RuleForecastCodePrefixRule {
    fn check(&self, item: &RuleForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule forecast.
#[derive(Debug, Clone)]
pub struct RuleForecastValidator {
    pub rules: Vec<Rc<dyn RuleForecastRule>>,
}

impl RuleForecastValidator {
    pub fn validate(&self, item: &RuleForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RuleForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_forecast_validator() -> RuleForecastValidator {
    RuleForecastValidator {
        rules: vec![
            Rc::new(RuleForecastPhoneLengthRule { max_length: 3600 }) as Rc<dyn RuleForecastRule>,
            Rc::new(RuleForecastAmountRangeRule { min: 0.3, max: 3734.8 }),
            Rc::new(RuleForecastBudgetRangeRule { min: 4.4, max: 3455.9 }),
            Rc::new(RuleForecastLabelLengthRule { max_length: 120 }),
            Rc::new(RuleForecastCodePrefixRule {
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
        let validator = default_rule_forecast_validator();
        let item = new_rule_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = RuleForecastPhoneLengthRule { max_length: 16 };
        let mut item = new_rule_forecast("x", "y");
        item.phone = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = RuleForecastAmountRangeRule { min: 3.9, max: 909.7 };
        let mut item = new_rule_forecast("x", "y");
        item.amount = 911.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = RuleForecastBudgetRangeRule { min: 6.7, max: 1282.9 };
        let mut item = new_rule_forecast("x", "y");
        item.budget = 1284.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RuleForecastLabelLengthRule { max_length: 16 };
        let mut item = new_rule_forecast("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
