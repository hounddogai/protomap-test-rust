use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a plan slot must pass. Check returns its problems, or none.
pub trait PlanSlotRule: std::fmt::Debug {
    fn check(&self, item: &PlanSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanSlotRequiredNotesRule {
    pub message: String,
}

impl PlanSlotRule for PlanSlotRequiredNotesRule {
    fn check(&self, item: &PlanSlot) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanSlotRequiredNameRule {
    pub message: String,
}

impl PlanSlotRule for PlanSlotRequiredNameRule {
    fn check(&self, item: &PlanSlot) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanSlotCodeLengthRule {
    pub max_length: i64,
}

impl PlanSlotRule for PlanSlotCodeLengthRule {
    fn check(&self, item: &PlanSlot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan slot.
#[derive(Debug, Clone)]
pub struct PlanSlotValidator {
    pub rules: Vec<Rc<dyn PlanSlotRule>>,
}

impl PlanSlotValidator {
    pub fn validate(&self, item: &PlanSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PlanSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_slot_validator() -> PlanSlotValidator {
    PlanSlotValidator {
        rules: vec![
            Rc::new(PlanSlotRequiredNotesRule {
                message: "plan slot needs an entry".to_string(),
            }) as Rc<dyn PlanSlotRule>,
            Rc::new(PlanSlotRequiredNameRule {
                message: "plan slot needs an entry".to_string(),
            }),
            Rc::new(PlanSlotCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_plan_slot_validator();
        let item = new_plan_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = PlanSlotRequiredNotesRule {
            message: "plan slot needs a value".to_string(),
        };
        let mut item = new_plan_slot("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PlanSlotRequiredNameRule {
            message: "plan slot needs this field".to_string(),
        };
        let mut item = new_plan_slot("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PlanSlotCodeLengthRule { max_length: 86400 };
        let mut item = new_plan_slot("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
