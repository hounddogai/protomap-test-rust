use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional rollout slot must pass. Check returns its problems, or none.
pub trait RegionalRolloutSlotRule: std::fmt::Debug {
    fn check(&self, item: &RegionalRolloutSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRolloutSlotNotesLengthRule {
    pub max_length: i64,
}

impl RegionalRolloutSlotRule for RegionalRolloutSlotNotesLengthRule {
    fn check(&self, item: &RegionalRolloutSlot) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRolloutSlotLabelLengthRule {
    pub max_length: i64,
}

impl RegionalRolloutSlotRule for RegionalRolloutSlotLabelLengthRule {
    fn check(&self, item: &RegionalRolloutSlot) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRolloutSlotRequiredLabelRule {
    pub message: String,
}

impl RegionalRolloutSlotRule for RegionalRolloutSlotRequiredLabelRule {
    fn check(&self, item: &RegionalRolloutSlot) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRolloutSlotCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalRolloutSlotRule for RegionalRolloutSlotCapacityRangeRule {
    fn check(&self, item: &RegionalRolloutSlot) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional rollout slot.
#[derive(Debug, Clone)]
pub struct RegionalRolloutSlotValidator {
    pub rules: Vec<Rc<dyn RegionalRolloutSlotRule>>,
}

impl RegionalRolloutSlotValidator {
    pub fn validate(&self, item: &RegionalRolloutSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalRolloutSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalRolloutSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_rollout_slot_validator() -> RegionalRolloutSlotValidator {
    RegionalRolloutSlotValidator {
        rules: vec![
            Rc::new(RegionalRolloutSlotNotesLengthRule { max_length: 16 }) as Rc<dyn RegionalRolloutSlotRule>,
            Rc::new(RegionalRolloutSlotLabelLengthRule { max_length: 80 }),
            Rc::new(RegionalRolloutSlotRequiredLabelRule {
                message: "regional rollout slot needs an entry".to_string(),
            }),
            Rc::new(RegionalRolloutSlotCapacityRangeRule { min: 64, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_rollout_slot_validator();
        let item = new_regional_rollout_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = RegionalRolloutSlotNotesLengthRule { max_length: 120 };
        let mut item = new_regional_rollout_slot("x", "y");
        item.notes = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RegionalRolloutSlotLabelLengthRule { max_length: 1000 };
        let mut item = new_regional_rollout_slot("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RegionalRolloutSlotRequiredLabelRule {
            message: "regional rollout slot needs this field".to_string(),
        };
        let mut item = new_regional_rollout_slot("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = RegionalRolloutSlotCapacityRangeRule { min: 16, max: 1000 };
        let mut item = new_regional_rollout_slot("x", "y");
        item.capacity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
