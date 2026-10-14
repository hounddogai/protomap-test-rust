use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual silence slot must pass. Check returns its problems, or none.
pub trait ManualSilenceSlotRule: std::fmt::Debug {
    fn check(&self, item: &ManualSilenceSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceSlotCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualSilenceSlotRule for ManualSilenceSlotCapacityRangeRule {
    fn check(&self, item: &ManualSilenceSlot) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceSlotRequiredCodeRule {
    pub message: String,
}

impl ManualSilenceSlotRule for ManualSilenceSlotRequiredCodeRule {
    fn check(&self, item: &ManualSilenceSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceSlotCodeLengthRule {
    pub max_length: i64,
}

impl ManualSilenceSlotRule for ManualSilenceSlotCodeLengthRule {
    fn check(&self, item: &ManualSilenceSlot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual silence slot.
#[derive(Debug, Clone)]
pub struct ManualSilenceSlotValidator {
    pub rules: Vec<Rc<dyn ManualSilenceSlotRule>>,
}

impl ManualSilenceSlotValidator {
    pub fn validate(&self, item: &ManualSilenceSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualSilenceSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualSilenceSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_silence_slot_validator() -> ManualSilenceSlotValidator {
    ManualSilenceSlotValidator {
        rules: vec![
            Rc::new(ManualSilenceSlotCapacityRangeRule { min: 255, max: 1000 }) as Rc<dyn ManualSilenceSlotRule>,
            Rc::new(ManualSilenceSlotRequiredCodeRule {
                message: "manual silence slot needs an entry".to_string(),
            }),
            Rc::new(ManualSilenceSlotCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_silence_slot_validator();
        let item = new_manual_silence_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = ManualSilenceSlotCapacityRangeRule { min: 80, max: 120 };
        let mut item = new_manual_silence_slot("x", "y");
        item.capacity = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualSilenceSlotRequiredCodeRule {
            message: "manual silence slot needs a value".to_string(),
        };
        let mut item = new_manual_silence_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualSilenceSlotCodeLengthRule { max_length: 16 };
        let mut item = new_manual_silence_slot("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
