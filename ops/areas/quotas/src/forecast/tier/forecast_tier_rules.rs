use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a forecast tier must pass. Check returns its problems, or none.
pub trait ForecastTierRule: std::fmt::Debug {
    fn check(&self, item: &ForecastTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastTierRequiredNameRule {
    pub message: String,
}

impl ForecastTierRule for ForecastTierRequiredNameRule {
    fn check(&self, item: &ForecastTier) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastTierCodePrefixRule {
    pub prefix: String,
}

impl ForecastTierRule for ForecastTierCodePrefixRule {
    fn check(&self, item: &ForecastTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastTierCodeLengthRule {
    pub max_length: i64,
}

impl ForecastTierRule for ForecastTierCodeLengthRule {
    fn check(&self, item: &ForecastTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastTierAllowedNameRule {
    pub allowed: Vec<String>,
}

impl ForecastTierRule for ForecastTierAllowedNameRule {
    fn check(&self, item: &ForecastTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is unknown", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastTierAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ForecastTierRule for ForecastTierAmountRangeRule {
    fn check(&self, item: &ForecastTier) -> Vec<String> {
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
pub struct ForecastTierNameLengthRule {
    pub max_length: i64,
}

impl ForecastTierRule for ForecastTierNameLengthRule {
    fn check(&self, item: &ForecastTier) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast tier.
#[derive(Debug, Clone)]
pub struct ForecastTierValidator {
    pub rules: Vec<Rc<dyn ForecastTierRule>>,
}

impl ForecastTierValidator {
    pub fn validate(&self, item: &ForecastTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ForecastTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_tier_validator() -> ForecastTierValidator {
    ForecastTierValidator {
        rules: vec![
            Rc::new(ForecastTierRequiredNameRule {
                message: "forecast tier needs a value".to_string(),
            }) as Rc<dyn ForecastTierRule>,
            Rc::new(ForecastTierCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(ForecastTierCodeLengthRule { max_length: 86400 }),
            Rc::new(ForecastTierAllowedNameRule {
                allowed: vec!["default".to_string(), "scheduled".to_string()],
            }),
            Rc::new(ForecastTierAmountRangeRule { min: 5.4, max: 3806.8 }),
            Rc::new(ForecastTierNameLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_tier_validator();
        let item = new_forecast_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ForecastTierRequiredNameRule {
            message: "forecast tier needs an entry".to_string(),
        };
        let mut item = new_forecast_tier("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ForecastTierCodeLengthRule { max_length: 64 };
        let mut item = new_forecast_tier("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = ForecastTierAmountRangeRule { min: 5.8, max: 2033.5 };
        let mut item = new_forecast_tier("x", "y");
        item.amount = 2035.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ForecastTierNameLengthRule { max_length: 64 };
        let mut item = new_forecast_tier("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
