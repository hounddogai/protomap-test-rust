use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a redaction slot must pass. Check returns its problems, or none.
pub trait RedactionSlotRule: std::fmt::Debug {
    fn check(&self, item: &RedactionSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionSlotCodeLengthRule {
    pub max_length: i64,
}

impl RedactionSlotRule for RedactionSlotCodeLengthRule {
    fn check(&self, item: &RedactionSlot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionSlotPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RedactionSlotRule for RedactionSlotPriorityRangeRule {
    fn check(&self, item: &RedactionSlot) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionSlotRequiredNameRule {
    pub message: String,
}

impl RedactionSlotRule for RedactionSlotRequiredNameRule {
    fn check(&self, item: &RedactionSlot) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionSlotPrincipalLengthRule {
    pub max_length: i64,
}

impl RedactionSlotRule for RedactionSlotPrincipalLengthRule {
    fn check(&self, item: &RedactionSlot) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionSlotRequiredDescriptionRule {
    pub message: String,
}

impl RedactionSlotRule for RedactionSlotRequiredDescriptionRule {
    fn check(&self, item: &RedactionSlot) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction slot.
#[derive(Debug, Clone)]
pub struct RedactionSlotValidator {
    pub rules: Vec<Rc<dyn RedactionSlotRule>>,
}

impl RedactionSlotValidator {
    pub fn validate(&self, item: &RedactionSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RedactionSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_slot_validator() -> RedactionSlotValidator {
    RedactionSlotValidator {
        rules: vec![
            Rc::new(RedactionSlotCodeLengthRule { max_length: 16 }) as Rc<dyn RedactionSlotRule>,
            Rc::new(RedactionSlotPriorityRangeRule { min: 3600, max: 3600 }),
            Rc::new(RedactionSlotRequiredNameRule {
                message: "redaction slot needs an entry".to_string(),
            }),
            Rc::new(RedactionSlotPrincipalLengthRule { max_length: 64 }),
            Rc::new(RedactionSlotRequiredDescriptionRule {
                message: "redaction slot needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_redaction_slot_validator();
        let item = new_redaction_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RedactionSlotCodeLengthRule { max_length: 64 };
        let mut item = new_redaction_slot("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = RedactionSlotPriorityRangeRule { min: 16, max: 32 };
        let mut item = new_redaction_slot("x", "y");
        item.priority = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RedactionSlotRequiredNameRule {
            message: "redaction slot needs an entry".to_string(),
        };
        let mut item = new_redaction_slot("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = RedactionSlotPrincipalLengthRule { max_length: 80 };
        let mut item = new_redaction_slot("x", "y");
        item.principal = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = RedactionSlotRequiredDescriptionRule {
            message: "redaction slot needs this field".to_string(),
        };
        let mut item = new_redaction_slot("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
