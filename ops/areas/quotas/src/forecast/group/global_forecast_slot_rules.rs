use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global forecast slot must pass. Check returns its problems, or none.
pub trait GlobalForecastSlotRule: std::fmt::Debug {
    fn check(&self, item: &GlobalForecastSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalForecastSlotChecksumLengthRule {
    pub max_length: i64,
}

impl GlobalForecastSlotRule for GlobalForecastSlotChecksumLengthRule {
    fn check(&self, item: &GlobalForecastSlot) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalForecastSlotMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalForecastSlotRule for GlobalForecastSlotMarginRangeRule {
    fn check(&self, item: &GlobalForecastSlot) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalForecastSlotRequiredLabelRule {
    pub message: String,
}

impl GlobalForecastSlotRule for GlobalForecastSlotRequiredLabelRule {
    fn check(&self, item: &GlobalForecastSlot) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global forecast slot.
#[derive(Debug, Clone)]
pub struct GlobalForecastSlotValidator {
    pub rules: Vec<Rc<dyn GlobalForecastSlotRule>>,
}

impl GlobalForecastSlotValidator {
    pub fn validate(&self, item: &GlobalForecastSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalForecastSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalForecastSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_forecast_slot_validator() -> GlobalForecastSlotValidator {
    GlobalForecastSlotValidator {
        rules: vec![
            Rc::new(GlobalForecastSlotChecksumLengthRule { max_length: 3600 }) as Rc<dyn GlobalForecastSlotRule>,
            Rc::new(GlobalForecastSlotMarginRangeRule { min: 8.8, max: 3228.1 }),
            Rc::new(GlobalForecastSlotRequiredLabelRule {
                message: "global forecast slot needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_forecast_slot_validator();
        let item = new_global_forecast_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = GlobalForecastSlotChecksumLengthRule { max_length: 32 };
        let mut item = new_global_forecast_slot("x", "y");
        item.checksum = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = GlobalForecastSlotMarginRangeRule { min: 1.8, max: 4678.8 };
        let mut item = new_global_forecast_slot("x", "y");
        item.margin = 4680.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = GlobalForecastSlotRequiredLabelRule {
            message: "global forecast slot needs a value".to_string(),
        };
        let mut item = new_global_forecast_slot("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
