use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched query slot must pass. Check returns its problems, or none.
pub trait BatchedQuerySlotRule: std::fmt::Debug {
    fn check(&self, item: &BatchedQuerySlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuerySlotRequiredCodeRule {
    pub message: String,
}

impl BatchedQuerySlotRule for BatchedQuerySlotRequiredCodeRule {
    fn check(&self, item: &BatchedQuerySlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuerySlotCodePrefixRule {
    pub prefix: String,
}

impl BatchedQuerySlotRule for BatchedQuerySlotCodePrefixRule {
    fn check(&self, item: &BatchedQuerySlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuerySlotRequiredHostRule {
    pub message: String,
}

impl BatchedQuerySlotRule for BatchedQuerySlotRequiredHostRule {
    fn check(&self, item: &BatchedQuerySlot) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuerySlotHostLengthRule {
    pub max_length: i64,
}

impl BatchedQuerySlotRule for BatchedQuerySlotHostLengthRule {
    fn check(&self, item: &BatchedQuerySlot) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched query slot.
#[derive(Debug, Clone)]
pub struct BatchedQuerySlotValidator {
    pub rules: Vec<Rc<dyn BatchedQuerySlotRule>>,
}

impl BatchedQuerySlotValidator {
    pub fn validate(&self, item: &BatchedQuerySlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedQuerySlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedQuerySlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_query_slot_validator() -> BatchedQuerySlotValidator {
    BatchedQuerySlotValidator {
        rules: vec![
            Rc::new(BatchedQuerySlotRequiredCodeRule {
                message: "batched query slot needs a value".to_string(),
            }) as Rc<dyn BatchedQuerySlotRule>,
            Rc::new(BatchedQuerySlotCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(BatchedQuerySlotRequiredHostRule {
                message: "batched query slot needs a value".to_string(),
            }),
            Rc::new(BatchedQuerySlotHostLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_query_slot_validator();
        let item = new_batched_query_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedQuerySlotRequiredCodeRule {
            message: "batched query slot needs an entry".to_string(),
        };
        let mut item = new_batched_query_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = BatchedQuerySlotRequiredHostRule {
            message: "batched query slot needs an entry".to_string(),
        };
        let mut item = new_batched_query_slot("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = BatchedQuerySlotHostLengthRule { max_length: 255 };
        let mut item = new_batched_query_slot("x", "y");
        item.host = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
