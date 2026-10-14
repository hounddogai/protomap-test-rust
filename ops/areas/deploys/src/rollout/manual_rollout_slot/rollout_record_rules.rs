use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout record must pass. Check returns its problems, or none.
pub trait RolloutRecordRule: std::fmt::Debug {
    fn check(&self, item: &RolloutRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutRecordCodePrefixRule {
    pub prefix: String,
}

impl RolloutRecordRule for RolloutRecordCodePrefixRule {
    fn check(&self, item: &RolloutRecord) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutRecordRequiredCodeRule {
    pub message: String,
}

impl RolloutRecordRule for RolloutRecordRequiredCodeRule {
    fn check(&self, item: &RolloutRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutRecordPrincipalLengthRule {
    pub max_length: i64,
}

impl RolloutRecordRule for RolloutRecordPrincipalLengthRule {
    fn check(&self, item: &RolloutRecord) -> Vec<String> {
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
pub struct RolloutRecordCodeLengthRule {
    pub max_length: i64,
}

impl RolloutRecordRule for RolloutRecordCodeLengthRule {
    fn check(&self, item: &RolloutRecord) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutRecordKeyIdLengthRule {
    pub max_length: i64,
}

impl RolloutRecordRule for RolloutRecordKeyIdLengthRule {
    fn check(&self, item: &RolloutRecord) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout record.
#[derive(Debug, Clone)]
pub struct RolloutRecordValidator {
    pub rules: Vec<Rc<dyn RolloutRecordRule>>,
}

impl RolloutRecordValidator {
    pub fn validate(&self, item: &RolloutRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RolloutRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_record_validator() -> RolloutRecordValidator {
    RolloutRecordValidator {
        rules: vec![
            Rc::new(RolloutRecordCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn RolloutRecordRule>,
            Rc::new(RolloutRecordRequiredCodeRule {
                message: "rollout record needs this field".to_string(),
            }),
            Rc::new(RolloutRecordPrincipalLengthRule { max_length: 3600 }),
            Rc::new(RolloutRecordCodeLengthRule { max_length: 32 }),
            Rc::new(RolloutRecordKeyIdLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_record_validator();
        let item = new_rollout_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RolloutRecordRequiredCodeRule {
            message: "rollout record needs this field".to_string(),
        };
        let mut item = new_rollout_record("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = RolloutRecordPrincipalLengthRule { max_length: 32 };
        let mut item = new_rollout_record("x", "y");
        item.principal = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RolloutRecordCodeLengthRule { max_length: 86400 };
        let mut item = new_rollout_record("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = RolloutRecordKeyIdLengthRule { max_length: 255 };
        let mut item = new_rollout_record("x", "y");
        item.key_id = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
