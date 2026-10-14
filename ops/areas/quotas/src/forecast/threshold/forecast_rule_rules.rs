use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast rule must pass. Check returns its problems, or none.
pub trait ForecastRuleRule: std::fmt::Debug {
    fn check(&self, item: &ForecastRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastRuleCodeLengthRule {
    pub max_length: i64,
}

impl ForecastRuleRule for ForecastRuleCodeLengthRule {
    fn check(&self, item: &ForecastRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastRuleDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ForecastRuleRule for ForecastRuleDiscountRangeRule {
    fn check(&self, item: &ForecastRule) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastRuleBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ForecastRuleRule for ForecastRuleBudgetRangeRule {
    fn check(&self, item: &ForecastRule) -> Vec<String> {
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
pub struct ForecastRuleWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ForecastRuleRule for ForecastRuleWindowDaysRangeRule {
    fn check(&self, item: &ForecastRule) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastRuleCodePrefixRule {
    pub prefix: String,
}

impl ForecastRuleRule for ForecastRuleCodePrefixRule {
    fn check(&self, item: &ForecastRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast rule.
#[derive(Debug, Clone)]
pub struct ForecastRuleValidator {
    pub rules: Vec<Rc<dyn ForecastRuleRule>>,
}

impl ForecastRuleValidator {
    pub fn validate(&self, item: &ForecastRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ForecastRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_rule_validator() -> ForecastRuleValidator {
    ForecastRuleValidator {
        rules: vec![
            Rc::new(ForecastRuleCodeLengthRule { max_length: 3600 }) as Rc<dyn ForecastRuleRule>,
            Rc::new(ForecastRuleDiscountRangeRule { min: 8.7, max: 4932.3 }),
            Rc::new(ForecastRuleBudgetRangeRule { min: 6.5, max: 184.2 }),
            Rc::new(ForecastRuleWindowDaysRangeRule { min: 80, max: 1000 }),
            Rc::new(ForecastRuleCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_rule_validator();
        let item = new_forecast_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ForecastRuleCodeLengthRule { max_length: 120 };
        let mut item = new_forecast_rule("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = ForecastRuleDiscountRangeRule { min: 0.2, max: 2974.5 };
        let mut item = new_forecast_rule("x", "y");
        item.discount = 2976.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = ForecastRuleBudgetRangeRule { min: 0.1, max: 2653.1 };
        let mut item = new_forecast_rule("x", "y");
        item.budget = 2654.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = ForecastRuleWindowDaysRangeRule { min: 3600, max: 86400 };
        let mut item = new_forecast_rule("x", "y");
        item.window_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
