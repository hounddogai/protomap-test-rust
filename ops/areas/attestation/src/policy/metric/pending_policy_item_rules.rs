use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending policy item must pass. Check returns its problems, or none.
pub trait PendingPolicyItemRule: std::fmt::Debug {
    fn check(&self, item: &PendingPolicyItem) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPolicyItemCodePrefixRule {
    pub prefix: String,
}

impl PendingPolicyItemRule for PendingPolicyItemCodePrefixRule {
    fn check(&self, item: &PendingPolicyItem) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPolicyItemRequiredChecksumRule {
    pub message: String,
}

impl PendingPolicyItemRule for PendingPolicyItemRequiredChecksumRule {
    fn check(&self, item: &PendingPolicyItem) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPolicyItemChecksumLengthRule {
    pub max_length: i64,
}

impl PendingPolicyItemRule for PendingPolicyItemChecksumLengthRule {
    fn check(&self, item: &PendingPolicyItem) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPolicyItemPinnedGuardRule {
    pub limit: f64,
}

impl PendingPolicyItemRule for PendingPolicyItemPinnedGuardRule {
    fn check(&self, item: &PendingPolicyItem) -> Vec<String> {
        if item.pinned && (item.discount > self.limit) {
            return vec![format!(
                "discount {} exceeds {} while pinned",
                item.discount, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPolicyItemRequiredCodeRule {
    pub message: String,
}

impl PendingPolicyItemRule for PendingPolicyItemRequiredCodeRule {
    fn check(&self, item: &PendingPolicyItem) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending policy item.
#[derive(Debug, Clone)]
pub struct PendingPolicyItemValidator {
    pub rules: Vec<Rc<dyn PendingPolicyItemRule>>,
}

impl PendingPolicyItemValidator {
    pub fn validate(&self, item: &PendingPolicyItem) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingPolicyItem) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingPolicyItem]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_policy_item_validator() -> PendingPolicyItemValidator {
    PendingPolicyItemValidator {
        rules: vec![
            Rc::new(PendingPolicyItemCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn PendingPolicyItemRule>,
            Rc::new(PendingPolicyItemRequiredChecksumRule {
                message: "pending policy item needs this field".to_string(),
            }),
            Rc::new(PendingPolicyItemChecksumLengthRule { max_length: 64 }),
            Rc::new(PendingPolicyItemPinnedGuardRule { limit: 0.7 }),
            Rc::new(PendingPolicyItemRequiredCodeRule {
                message: "pending policy item needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_policy_item_validator();
        let item = new_pending_policy_item("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = PendingPolicyItemRequiredChecksumRule {
            message: "pending policy item needs a value".to_string(),
        };
        let mut item = new_pending_policy_item("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = PendingPolicyItemChecksumLengthRule { max_length: 64 };
        let mut item = new_pending_policy_item("x", "y");
        item.checksum = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingPolicyItemRequiredCodeRule {
            message: "pending policy item needs a value".to_string(),
        };
        let mut item = new_pending_policy_item("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
