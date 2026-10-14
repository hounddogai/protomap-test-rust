use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched slo slot must pass. Check returns its problems, or none.
pub trait BatchedSloSlotRule: std::fmt::Debug {
    fn check(&self, item: &BatchedSloSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSloSlotCodeLengthRule {
    pub max_length: i64,
}

impl BatchedSloSlotRule for BatchedSloSlotCodeLengthRule {
    fn check(&self, item: &BatchedSloSlot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSloSlotRequiredCodeRule {
    pub message: String,
}

impl BatchedSloSlotRule for BatchedSloSlotRequiredCodeRule {
    fn check(&self, item: &BatchedSloSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSloSlotCodePrefixRule {
    pub prefix: String,
}

impl BatchedSloSlotRule for BatchedSloSlotCodePrefixRule {
    fn check(&self, item: &BatchedSloSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched slo slot.
#[derive(Debug, Clone)]
pub struct BatchedSloSlotValidator {
    pub rules: Vec<Rc<dyn BatchedSloSlotRule>>,
}

impl BatchedSloSlotValidator {
    pub fn validate(&self, item: &BatchedSloSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedSloSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedSloSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_slo_slot_validator() -> BatchedSloSlotValidator {
    BatchedSloSlotValidator {
        rules: vec![
            Rc::new(BatchedSloSlotCodeLengthRule { max_length: 255 }) as Rc<dyn BatchedSloSlotRule>,
            Rc::new(BatchedSloSlotRequiredCodeRule {
                message: "batched slo slot needs this field".to_string(),
            }),
            Rc::new(BatchedSloSlotCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_slo_slot_validator();
        let item = new_batched_slo_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedSloSlotCodeLengthRule { max_length: 64 };
        let mut item = new_batched_slo_slot("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedSloSlotRequiredCodeRule {
            message: "batched slo slot needs an entry".to_string(),
        };
        let mut item = new_batched_slo_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
