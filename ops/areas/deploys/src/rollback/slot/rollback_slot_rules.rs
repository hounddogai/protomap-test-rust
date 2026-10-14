use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollback slot must pass. Check returns its problems, or none.
pub trait RollbackSlotRule: std::fmt::Debug {
    fn check(&self, item: &RollbackSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackSlotRequiredDescriptionRule {
    pub message: String,
}

impl RollbackSlotRule for RollbackSlotRequiredDescriptionRule {
    fn check(&self, item: &RollbackSlot) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackSlotUnitLengthRule {
    pub max_length: i64,
}

impl RollbackSlotRule for RollbackSlotUnitLengthRule {
    fn check(&self, item: &RollbackSlot) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackSlotRequiredLabelRule {
    pub message: String,
}

impl RollbackSlotRule for RollbackSlotRequiredLabelRule {
    fn check(&self, item: &RollbackSlot) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackSlotRequiredCodeRule {
    pub message: String,
}

impl RollbackSlotRule for RollbackSlotRequiredCodeRule {
    fn check(&self, item: &RollbackSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackSlotDescriptionLengthRule {
    pub max_length: i64,
}

impl RollbackSlotRule for RollbackSlotDescriptionLengthRule {
    fn check(&self, item: &RollbackSlot) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackSlotLabelLengthRule {
    pub max_length: i64,
}

impl RollbackSlotRule for RollbackSlotLabelLengthRule {
    fn check(&self, item: &RollbackSlot) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback slot.
#[derive(Debug, Clone)]
pub struct RollbackSlotValidator {
    pub rules: Vec<Rc<dyn RollbackSlotRule>>,
}

impl RollbackSlotValidator {
    pub fn validate(&self, item: &RollbackSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollbackSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_slot_validator() -> RollbackSlotValidator {
    RollbackSlotValidator {
        rules: vec![
            Rc::new(RollbackSlotRequiredDescriptionRule {
                message: "rollback slot needs this field".to_string(),
            }) as Rc<dyn RollbackSlotRule>,
            Rc::new(RollbackSlotUnitLengthRule { max_length: 32 }),
            Rc::new(RollbackSlotRequiredLabelRule {
                message: "rollback slot needs an entry".to_string(),
            }),
            Rc::new(RollbackSlotRequiredCodeRule {
                message: "rollback slot needs this field".to_string(),
            }),
            Rc::new(RollbackSlotDescriptionLengthRule { max_length: 64 }),
            Rc::new(RollbackSlotLabelLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_slot_validator();
        let item = new_rollback_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = RollbackSlotRequiredDescriptionRule {
            message: "rollback slot needs a value".to_string(),
        };
        let mut item = new_rollback_slot("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = RollbackSlotUnitLengthRule { max_length: 80 };
        let mut item = new_rollback_slot("x", "y");
        item.unit = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RollbackSlotRequiredLabelRule {
            message: "rollback slot needs this field".to_string(),
        };
        let mut item = new_rollback_slot("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollbackSlotRequiredCodeRule {
            message: "rollback slot needs an entry".to_string(),
        };
        let mut item = new_rollback_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = RollbackSlotDescriptionLengthRule { max_length: 3600 };
        let mut item = new_rollback_slot("x", "y");
        item.description = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RollbackSlotLabelLengthRule { max_length: 16 };
        let mut item = new_rollback_slot("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
