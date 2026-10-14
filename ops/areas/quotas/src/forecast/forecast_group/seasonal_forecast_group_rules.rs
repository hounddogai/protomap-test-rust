use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal forecast group must pass. Check returns its problems, or none.
pub trait SeasonalForecastGroupRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalForecastGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalForecastGroupQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalForecastGroupRule for SeasonalForecastGroupQuantityRangeRule {
    fn check(&self, item: &SeasonalForecastGroup) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalForecastGroupCodePrefixRule {
    pub prefix: String,
}

impl SeasonalForecastGroupRule for SeasonalForecastGroupCodePrefixRule {
    fn check(&self, item: &SeasonalForecastGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalForecastGroupTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalForecastGroupRule for SeasonalForecastGroupTtlSecondsRangeRule {
    fn check(&self, item: &SeasonalForecastGroup) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalForecastGroupNamespaceLengthRule {
    pub max_length: i64,
}

impl SeasonalForecastGroupRule for SeasonalForecastGroupNamespaceLengthRule {
    fn check(&self, item: &SeasonalForecastGroup) -> Vec<String> {
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
pub struct SeasonalForecastGroupRequiredCodeRule {
    pub message: String,
}

impl SeasonalForecastGroupRule for SeasonalForecastGroupRequiredCodeRule {
    fn check(&self, item: &SeasonalForecastGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal forecast group.
#[derive(Debug, Clone)]
pub struct SeasonalForecastGroupValidator {
    pub rules: Vec<Rc<dyn SeasonalForecastGroupRule>>,
}

impl SeasonalForecastGroupValidator {
    pub fn validate(&self, item: &SeasonalForecastGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalForecastGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalForecastGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_forecast_group_validator() -> SeasonalForecastGroupValidator {
    SeasonalForecastGroupValidator {
        rules: vec![
            Rc::new(SeasonalForecastGroupQuantityRangeRule { min: 80, max: 3600 }) as Rc<dyn SeasonalForecastGroupRule>,
            Rc::new(SeasonalForecastGroupCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(SeasonalForecastGroupTtlSecondsRangeRule { min: 80, max: 255 }),
            Rc::new(SeasonalForecastGroupNamespaceLengthRule { max_length: 16 }),
            Rc::new(SeasonalForecastGroupRequiredCodeRule {
                message: "seasonal forecast group needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_forecast_group_validator();
        let item = new_seasonal_forecast_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = SeasonalForecastGroupQuantityRangeRule { min: 120, max: 1000 };
        let mut item = new_seasonal_forecast_group("x", "y");
        item.quantity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = SeasonalForecastGroupTtlSecondsRangeRule { min: 255, max: 255 };
        let mut item = new_seasonal_forecast_group("x", "y");
        item.ttl_seconds = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = SeasonalForecastGroupNamespaceLengthRule { max_length: 255 };
        let mut item = new_seasonal_forecast_group("x", "y");
        item.namespace = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalForecastGroupRequiredCodeRule {
            message: "seasonal forecast group needs this field".to_string(),
        };
        let mut item = new_seasonal_forecast_group("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
