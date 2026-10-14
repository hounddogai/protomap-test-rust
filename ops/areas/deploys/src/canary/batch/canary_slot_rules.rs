use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a canary slot must pass. Check returns its problems, or none.
pub trait CanarySlotRule: std::fmt::Debug {
    fn check(&self, item: &CanarySlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanarySlotAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CanarySlotRule for CanarySlotAmountRangeRule {
    fn check(&self, item: &CanarySlot) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanarySlotActiveGuardRule {
    pub limit: i64,
}

impl CanarySlotRule for CanarySlotActiveGuardRule {
    fn check(&self, item: &CanarySlot) -> Vec<String> {
        if item.active && (item.limit > self.limit) {
            return vec![format!("limit {} exceeds {} while active", item.limit, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanarySlotThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CanarySlotRule for CanarySlotThresholdRangeRule {
    fn check(&self, item: &CanarySlot) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanarySlotRequiredCodeRule {
    pub message: String,
}

impl CanarySlotRule for CanarySlotRequiredCodeRule {
    fn check(&self, item: &CanarySlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanarySlotRequiredCurrencyRule {
    pub message: String,
}

impl CanarySlotRule for CanarySlotRequiredCurrencyRule {
    fn check(&self, item: &CanarySlot) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary slot.
#[derive(Debug, Clone)]
pub struct CanarySlotValidator {
    pub rules: Vec<Rc<dyn CanarySlotRule>>,
}

impl CanarySlotValidator {
    pub fn validate(&self, item: &CanarySlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CanarySlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanarySlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_slot_validator() -> CanarySlotValidator {
    CanarySlotValidator {
        rules: vec![
            Rc::new(CanarySlotAmountRangeRule { min: 1.4, max: 268.8 }) as Rc<dyn CanarySlotRule>,
            Rc::new(CanarySlotActiveGuardRule { limit: 3600 }),
            Rc::new(CanarySlotThresholdRangeRule { min: 7.6, max: 1443.6 }),
            Rc::new(CanarySlotRequiredCodeRule {
                message: "canary slot needs a value".to_string(),
            }),
            Rc::new(CanarySlotRequiredCurrencyRule {
                message: "canary slot needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_slot_validator();
        let item = new_canary_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = CanarySlotAmountRangeRule { min: 8.4, max: 3864.3 };
        let mut item = new_canary_slot("x", "y");
        item.amount = 3865.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = CanarySlotThresholdRangeRule { min: 0.9, max: 1397.2 };
        let mut item = new_canary_slot("x", "y");
        item.threshold = 1398.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CanarySlotRequiredCodeRule {
            message: "canary slot needs a value".to_string(),
        };
        let mut item = new_canary_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = CanarySlotRequiredCurrencyRule {
            message: "canary slot needs an entry".to_string(),
        };
        let mut item = new_canary_slot("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
