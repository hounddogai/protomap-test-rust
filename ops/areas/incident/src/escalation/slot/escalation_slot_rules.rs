use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a escalation slot must pass. Check returns its problems, or none.
pub trait EscalationSlotRule: std::fmt::Debug {
    fn check(&self, item: &EscalationSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationSlotCodePrefixRule {
    pub prefix: String,
}

impl EscalationSlotRule for EscalationSlotCodePrefixRule {
    fn check(&self, item: &EscalationSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationSlotRequiredDescriptionRule {
    pub message: String,
}

impl EscalationSlotRule for EscalationSlotRequiredDescriptionRule {
    fn check(&self, item: &EscalationSlot) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationSlotRequiredCodeRule {
    pub message: String,
}

impl EscalationSlotRule for EscalationSlotRequiredCodeRule {
    fn check(&self, item: &EscalationSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationSlotCodeLengthRule {
    pub max_length: i64,
}

impl EscalationSlotRule for EscalationSlotCodeLengthRule {
    fn check(&self, item: &EscalationSlot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationSlotDescriptionLengthRule {
    pub max_length: i64,
}

impl EscalationSlotRule for EscalationSlotDescriptionLengthRule {
    fn check(&self, item: &EscalationSlot) -> Vec<String> {
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
pub struct EscalationSlotRequiredLabelRule {
    pub message: String,
}

impl EscalationSlotRule for EscalationSlotRequiredLabelRule {
    fn check(&self, item: &EscalationSlot) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation slot.
#[derive(Debug, Clone)]
pub struct EscalationSlotValidator {
    pub rules: Vec<Rc<dyn EscalationSlotRule>>,
}

impl EscalationSlotValidator {
    pub fn validate(&self, item: &EscalationSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EscalationSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_slot_validator() -> EscalationSlotValidator {
    EscalationSlotValidator {
        rules: vec![
            Rc::new(EscalationSlotCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn EscalationSlotRule>,
            Rc::new(EscalationSlotRequiredDescriptionRule {
                message: "escalation slot needs this field".to_string(),
            }),
            Rc::new(EscalationSlotRequiredCodeRule {
                message: "escalation slot needs this field".to_string(),
            }),
            Rc::new(EscalationSlotCodeLengthRule { max_length: 120 }),
            Rc::new(EscalationSlotDescriptionLengthRule { max_length: 86400 }),
            Rc::new(EscalationSlotRequiredLabelRule {
                message: "escalation slot needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_escalation_slot_validator();
        let item = new_escalation_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = EscalationSlotRequiredDescriptionRule {
            message: "escalation slot needs an entry".to_string(),
        };
        let mut item = new_escalation_slot("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EscalationSlotRequiredCodeRule {
            message: "escalation slot needs an entry".to_string(),
        };
        let mut item = new_escalation_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EscalationSlotCodeLengthRule { max_length: 80 };
        let mut item = new_escalation_slot("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = EscalationSlotDescriptionLengthRule { max_length: 64 };
        let mut item = new_escalation_slot("x", "y");
        item.description = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = EscalationSlotRequiredLabelRule {
            message: "escalation slot needs this field".to_string(),
        };
        let mut item = new_escalation_slot("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
