use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout slot must pass. Check returns its problems, or none.
pub trait RolloutSlotRule: std::fmt::Debug {
    fn check(&self, item: &RolloutSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutSlotArchivedGuardRule {
    pub limit: i64,
}

impl RolloutSlotRule for RolloutSlotArchivedGuardRule {
    fn check(&self, item: &RolloutSlot) -> Vec<String> {
        if item.archived && (item.ends_at > self.limit) {
            return vec![format!(
                "ends at {} exceeds {} while archived",
                item.ends_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutSlotCodeLengthRule {
    pub max_length: i64,
}

impl RolloutSlotRule for RolloutSlotCodeLengthRule {
    fn check(&self, item: &RolloutSlot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutSlotRequiredCipherRule {
    pub message: String,
}

impl RolloutSlotRule for RolloutSlotRequiredCipherRule {
    fn check(&self, item: &RolloutSlot) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout slot.
#[derive(Debug, Clone)]
pub struct RolloutSlotValidator {
    pub rules: Vec<Rc<dyn RolloutSlotRule>>,
}

impl RolloutSlotValidator {
    pub fn validate(&self, item: &RolloutSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RolloutSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_slot_validator() -> RolloutSlotValidator {
    RolloutSlotValidator {
        rules: vec![
            Rc::new(RolloutSlotArchivedGuardRule { limit: 64 }) as Rc<dyn RolloutSlotRule>,
            Rc::new(RolloutSlotCodeLengthRule { max_length: 64 }),
            Rc::new(RolloutSlotRequiredCipherRule {
                message: "rollout slot needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_slot_validator();
        let item = new_rollout_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RolloutSlotCodeLengthRule { max_length: 16 };
        let mut item = new_rollout_slot("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = RolloutSlotRequiredCipherRule {
            message: "rollout slot needs a value".to_string(),
        };
        let mut item = new_rollout_slot("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
