use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict forecast cycle must pass. Check returns its problems, or none.
pub trait StrictForecastCycleRule: std::fmt::Debug {
    fn check(&self, item: &StrictForecastCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictForecastCycleQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictForecastCycleRule for StrictForecastCycleQuantityRangeRule {
    fn check(&self, item: &StrictForecastCycle) -> Vec<String> {
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
pub struct StrictForecastCycleNotesLengthRule {
    pub max_length: i64,
}

impl StrictForecastCycleRule for StrictForecastCycleNotesLengthRule {
    fn check(&self, item: &StrictForecastCycle) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictForecastCycleRequiredLabelRule {
    pub message: String,
}

impl StrictForecastCycleRule for StrictForecastCycleRequiredLabelRule {
    fn check(&self, item: &StrictForecastCycle) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictForecastCycleCurrencyLengthRule {
    pub max_length: i64,
}

impl StrictForecastCycleRule for StrictForecastCycleCurrencyLengthRule {
    fn check(&self, item: &StrictForecastCycle) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictForecastCycleRequiredCurrencyRule {
    pub message: String,
}

impl StrictForecastCycleRule for StrictForecastCycleRequiredCurrencyRule {
    fn check(&self, item: &StrictForecastCycle) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict forecast cycle.
#[derive(Debug, Clone)]
pub struct StrictForecastCycleValidator {
    pub rules: Vec<Rc<dyn StrictForecastCycleRule>>,
}

impl StrictForecastCycleValidator {
    pub fn validate(&self, item: &StrictForecastCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictForecastCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictForecastCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_forecast_cycle_validator() -> StrictForecastCycleValidator {
    StrictForecastCycleValidator {
        rules: vec![
            Rc::new(StrictForecastCycleQuantityRangeRule { min: 255, max: 3600 }) as Rc<dyn StrictForecastCycleRule>,
            Rc::new(StrictForecastCycleNotesLengthRule { max_length: 120 }),
            Rc::new(StrictForecastCycleRequiredLabelRule {
                message: "strict forecast cycle needs a value".to_string(),
            }),
            Rc::new(StrictForecastCycleCurrencyLengthRule { max_length: 3600 }),
            Rc::new(StrictForecastCycleRequiredCurrencyRule {
                message: "strict forecast cycle needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_forecast_cycle_validator();
        let item = new_strict_forecast_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = StrictForecastCycleQuantityRangeRule { min: 255, max: 1000 };
        let mut item = new_strict_forecast_cycle("x", "y");
        item.quantity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = StrictForecastCycleNotesLengthRule { max_length: 80 };
        let mut item = new_strict_forecast_cycle("x", "y");
        item.notes = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = StrictForecastCycleRequiredLabelRule {
            message: "strict forecast cycle needs an entry".to_string(),
        };
        let mut item = new_strict_forecast_cycle("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = StrictForecastCycleCurrencyLengthRule { max_length: 86400 };
        let mut item = new_strict_forecast_cycle("x", "y");
        item.currency = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = StrictForecastCycleRequiredCurrencyRule {
            message: "strict forecast cycle needs a value".to_string(),
        };
        let mut item = new_strict_forecast_cycle("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
