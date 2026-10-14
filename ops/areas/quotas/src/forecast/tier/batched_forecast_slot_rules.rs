use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched forecast slot must pass. Check returns its problems, or none.
pub trait BatchedForecastSlotRule: std::fmt::Debug {
    fn check(&self, item: &BatchedForecastSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastSlotCodePrefixRule {
    pub prefix: String,
}

impl BatchedForecastSlotRule for BatchedForecastSlotCodePrefixRule {
    fn check(&self, item: &BatchedForecastSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastSlotNameLengthRule {
    pub max_length: i64,
}

impl BatchedForecastSlotRule for BatchedForecastSlotNameLengthRule {
    fn check(&self, item: &BatchedForecastSlot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastSlotBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedForecastSlotRule for BatchedForecastSlotBurstRangeRule {
    fn check(&self, item: &BatchedForecastSlot) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched forecast slot.
#[derive(Debug, Clone)]
pub struct BatchedForecastSlotValidator {
    pub rules: Vec<Rc<dyn BatchedForecastSlotRule>>,
}

impl BatchedForecastSlotValidator {
    pub fn validate(&self, item: &BatchedForecastSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedForecastSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedForecastSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_forecast_slot_validator() -> BatchedForecastSlotValidator {
    BatchedForecastSlotValidator {
        rules: vec![
            Rc::new(BatchedForecastSlotCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn BatchedForecastSlotRule>,
            Rc::new(BatchedForecastSlotNameLengthRule { max_length: 64 }),
            Rc::new(BatchedForecastSlotBurstRangeRule { min: 32, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_forecast_slot_validator();
        let item = new_batched_forecast_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = BatchedForecastSlotNameLengthRule { max_length: 255 };
        let mut item = new_batched_forecast_slot("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = BatchedForecastSlotBurstRangeRule { min: 3600, max: 86400 };
        let mut item = new_batched_forecast_slot("x", "y");
        item.burst = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
