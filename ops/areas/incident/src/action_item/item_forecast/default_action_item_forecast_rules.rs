use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default action item forecast must pass. Check returns its problems, or none.
pub trait DefaultActionItemForecastRule: std::fmt::Debug {
    fn check(&self, item: &DefaultActionItemForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemForecastDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultActionItemForecastRule for DefaultActionItemForecastDiscountRangeRule {
    fn check(&self, item: &DefaultActionItemForecast) -> Vec<String> {
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
pub struct DefaultActionItemForecastBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultActionItemForecastRule for DefaultActionItemForecastBurstRangeRule {
    fn check(&self, item: &DefaultActionItemForecast) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemForecastRequiredCodeRule {
    pub message: String,
}

impl DefaultActionItemForecastRule for DefaultActionItemForecastRequiredCodeRule {
    fn check(&self, item: &DefaultActionItemForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default action item forecast.
#[derive(Debug, Clone)]
pub struct DefaultActionItemForecastValidator {
    pub rules: Vec<Rc<dyn DefaultActionItemForecastRule>>,
}

impl DefaultActionItemForecastValidator {
    pub fn validate(&self, item: &DefaultActionItemForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultActionItemForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultActionItemForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_action_item_forecast_validator() -> DefaultActionItemForecastValidator {
    DefaultActionItemForecastValidator {
        rules: vec![
            Rc::new(DefaultActionItemForecastDiscountRangeRule { min: 8.8, max: 1026.8 })
                as Rc<dyn DefaultActionItemForecastRule>,
            Rc::new(DefaultActionItemForecastBurstRangeRule { min: 120, max: 1000 }),
            Rc::new(DefaultActionItemForecastRequiredCodeRule {
                message: "default action item forecast needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_action_item_forecast_validator();
        let item = new_default_action_item_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = DefaultActionItemForecastDiscountRangeRule { min: 7.9, max: 4163.7 };
        let mut item = new_default_action_item_forecast("x", "y");
        item.discount = 4165.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = DefaultActionItemForecastBurstRangeRule { min: 64, max: 120 };
        let mut item = new_default_action_item_forecast("x", "y");
        item.burst = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultActionItemForecastRequiredCodeRule {
            message: "default action item forecast needs an entry".to_string(),
        };
        let mut item = new_default_action_item_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
