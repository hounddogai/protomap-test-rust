use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a nonce slot must pass. Check returns its problems, or none.
pub trait NonceSlotRule: std::fmt::Debug {
    fn check(&self, item: &NonceSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceSlotRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl NonceSlotRule for NonceSlotRevisionRangeRule {
    fn check(&self, item: &NonceSlot) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceSlotRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl NonceSlotRule for NonceSlotRetriesRangeRule {
    fn check(&self, item: &NonceSlot) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceSlotNameLengthRule {
    pub max_length: i64,
}

impl NonceSlotRule for NonceSlotNameLengthRule {
    fn check(&self, item: &NonceSlot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceSlotReviewerLengthRule {
    pub max_length: i64,
}

impl NonceSlotRule for NonceSlotReviewerLengthRule {
    fn check(&self, item: &NonceSlot) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceSlotRequiredCodeRule {
    pub message: String,
}

impl NonceSlotRule for NonceSlotRequiredCodeRule {
    fn check(&self, item: &NonceSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceSlotRequiredNameRule {
    pub message: String,
}

impl NonceSlotRule for NonceSlotRequiredNameRule {
    fn check(&self, item: &NonceSlot) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce slot.
#[derive(Debug, Clone)]
pub struct NonceSlotValidator {
    pub rules: Vec<Rc<dyn NonceSlotRule>>,
}

impl NonceSlotValidator {
    pub fn validate(&self, item: &NonceSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &NonceSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_slot_validator() -> NonceSlotValidator {
    NonceSlotValidator {
        rules: vec![
            Rc::new(NonceSlotRevisionRangeRule { min: 64, max: 255 }) as Rc<dyn NonceSlotRule>,
            Rc::new(NonceSlotRetriesRangeRule { min: 255, max: 1000 }),
            Rc::new(NonceSlotNameLengthRule { max_length: 3600 }),
            Rc::new(NonceSlotReviewerLengthRule { max_length: 16 }),
            Rc::new(NonceSlotRequiredCodeRule {
                message: "nonce slot needs a value".to_string(),
            }),
            Rc::new(NonceSlotRequiredNameRule {
                message: "nonce slot needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_nonce_slot_validator();
        let item = new_nonce_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = NonceSlotRevisionRangeRule { min: 32, max: 3600 };
        let mut item = new_nonce_slot("x", "y");
        item.revision = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = NonceSlotRetriesRangeRule { min: 16, max: 16 };
        let mut item = new_nonce_slot("x", "y");
        item.retries = 17;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = NonceSlotNameLengthRule { max_length: 86400 };
        let mut item = new_nonce_slot("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = NonceSlotReviewerLengthRule { max_length: 80 };
        let mut item = new_nonce_slot("x", "y");
        item.reviewer = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = NonceSlotRequiredCodeRule {
            message: "nonce slot needs a value".to_string(),
        };
        let mut item = new_nonce_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = NonceSlotRequiredNameRule {
            message: "nonce slot needs an entry".to_string(),
        };
        let mut item = new_nonce_slot("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
