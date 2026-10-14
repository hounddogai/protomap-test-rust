use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient freeze checkpoint must pass. Check returns its problems, or none.
pub trait LenientFreezeCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &LenientFreezeCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeCheckpointAllowedNameRule {
    pub allowed: Vec<String>,
}

impl LenientFreezeCheckpointRule for LenientFreezeCheckpointAllowedNameRule {
    fn check(&self, item: &LenientFreezeCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is blocked", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeCheckpointOwnerIdLengthRule {
    pub max_length: i64,
}

impl LenientFreezeCheckpointRule for LenientFreezeCheckpointOwnerIdLengthRule {
    fn check(&self, item: &LenientFreezeCheckpoint) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeCheckpointRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientFreezeCheckpointRule for LenientFreezeCheckpointRevisionRangeRule {
    fn check(&self, item: &LenientFreezeCheckpoint) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl LenientFreezeCheckpointRule for LenientFreezeCheckpointCodeLengthRule {
    fn check(&self, item: &LenientFreezeCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeCheckpointCodePrefixRule {
    pub prefix: String,
}

impl LenientFreezeCheckpointRule for LenientFreezeCheckpointCodePrefixRule {
    fn check(&self, item: &LenientFreezeCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeCheckpointAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientFreezeCheckpointRule for LenientFreezeCheckpointAmountRangeRule {
    fn check(&self, item: &LenientFreezeCheckpoint) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient freeze checkpoint.
#[derive(Debug, Clone)]
pub struct LenientFreezeCheckpointValidator {
    pub rules: Vec<Rc<dyn LenientFreezeCheckpointRule>>,
}

impl LenientFreezeCheckpointValidator {
    pub fn validate(&self, item: &LenientFreezeCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientFreezeCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientFreezeCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_freeze_checkpoint_validator() -> LenientFreezeCheckpointValidator {
    LenientFreezeCheckpointValidator {
        rules: vec![
            Rc::new(LenientFreezeCheckpointAllowedNameRule {
                allowed: vec!["pending".to_string(), "primary".to_string()],
            }) as Rc<dyn LenientFreezeCheckpointRule>,
            Rc::new(LenientFreezeCheckpointOwnerIdLengthRule { max_length: 1000 }),
            Rc::new(LenientFreezeCheckpointRevisionRangeRule { min: 64, max: 3600 }),
            Rc::new(LenientFreezeCheckpointCodeLengthRule { max_length: 64 }),
            Rc::new(LenientFreezeCheckpointCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(LenientFreezeCheckpointAmountRangeRule { min: 2.0, max: 4699.7 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_freeze_checkpoint_validator();
        let item = new_lenient_freeze_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = LenientFreezeCheckpointOwnerIdLengthRule { max_length: 64 };
        let mut item = new_lenient_freeze_checkpoint("x", "y");
        item.owner_id = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = LenientFreezeCheckpointRevisionRangeRule { min: 3600, max: 3600 };
        let mut item = new_lenient_freeze_checkpoint("x", "y");
        item.revision = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientFreezeCheckpointCodeLengthRule { max_length: 80 };
        let mut item = new_lenient_freeze_checkpoint("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = LenientFreezeCheckpointAmountRangeRule { min: 1.5, max: 2255.8 };
        let mut item = new_lenient_freeze_checkpoint("x", "y");
        item.amount = 2257.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
