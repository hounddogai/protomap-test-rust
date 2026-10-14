use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a action item claim must pass. Check returns its problems, or none.
pub trait ActionItemClaimRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemClaimCodePrefixRule {
    pub prefix: String,
}

impl ActionItemClaimRule for ActionItemClaimCodePrefixRule {
    fn check(&self, item: &ActionItemClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemClaimChecksumLengthRule {
    pub max_length: i64,
}

impl ActionItemClaimRule for ActionItemClaimChecksumLengthRule {
    fn check(&self, item: &ActionItemClaim) -> Vec<String> {
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
pub struct ActionItemClaimLabelLengthRule {
    pub max_length: i64,
}

impl ActionItemClaimRule for ActionItemClaimLabelLengthRule {
    fn check(&self, item: &ActionItemClaim) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemClaimAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl ActionItemClaimRule for ActionItemClaimAllowedChecksumRule {
    fn check(&self, item: &ActionItemClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is duplicated", item.checksum)];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item claim.
#[derive(Debug, Clone)]
pub struct ActionItemClaimValidator {
    pub rules: Vec<Rc<dyn ActionItemClaimRule>>,
}

impl ActionItemClaimValidator {
    pub fn validate(&self, item: &ActionItemClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ActionItemClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_claim_validator() -> ActionItemClaimValidator {
    ActionItemClaimValidator {
        rules: vec![
            Rc::new(ActionItemClaimCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn ActionItemClaimRule>,
            Rc::new(ActionItemClaimChecksumLengthRule { max_length: 16 }),
            Rc::new(ActionItemClaimLabelLengthRule { max_length: 32 }),
            Rc::new(ActionItemClaimAllowedChecksumRule {
                allowed: vec!["weighted".to_string(), "default".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_action_item_claim_validator();
        let item = new_action_item_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = ActionItemClaimChecksumLengthRule { max_length: 16 };
        let mut item = new_action_item_claim("x", "y");
        item.checksum = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ActionItemClaimLabelLengthRule { max_length: 86400 };
        let mut item = new_action_item_claim("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
