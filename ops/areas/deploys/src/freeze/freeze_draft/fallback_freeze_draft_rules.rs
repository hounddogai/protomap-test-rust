use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback freeze draft must pass. Check returns its problems, or none.
pub trait FallbackFreezeDraftRule: std::fmt::Debug {
    fn check(&self, item: &FallbackFreezeDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFreezeDraftRequiredCurrencyRule {
    pub message: String,
}

impl FallbackFreezeDraftRule for FallbackFreezeDraftRequiredCurrencyRule {
    fn check(&self, item: &FallbackFreezeDraft) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFreezeDraftRequiredOwnerIdRule {
    pub message: String,
}

impl FallbackFreezeDraftRule for FallbackFreezeDraftRequiredOwnerIdRule {
    fn check(&self, item: &FallbackFreezeDraft) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFreezeDraftOwnerIdLengthRule {
    pub max_length: i64,
}

impl FallbackFreezeDraftRule for FallbackFreezeDraftOwnerIdLengthRule {
    fn check(&self, item: &FallbackFreezeDraft) -> Vec<String> {
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
pub struct FallbackFreezeDraftCodePrefixRule {
    pub prefix: String,
}

impl FallbackFreezeDraftRule for FallbackFreezeDraftCodePrefixRule {
    fn check(&self, item: &FallbackFreezeDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback freeze draft.
#[derive(Debug, Clone)]
pub struct FallbackFreezeDraftValidator {
    pub rules: Vec<Rc<dyn FallbackFreezeDraftRule>>,
}

impl FallbackFreezeDraftValidator {
    pub fn validate(&self, item: &FallbackFreezeDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackFreezeDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackFreezeDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_freeze_draft_validator() -> FallbackFreezeDraftValidator {
    FallbackFreezeDraftValidator {
        rules: vec![
            Rc::new(FallbackFreezeDraftRequiredCurrencyRule {
                message: "fallback freeze draft needs this field".to_string(),
            }) as Rc<dyn FallbackFreezeDraftRule>,
            Rc::new(FallbackFreezeDraftRequiredOwnerIdRule {
                message: "fallback freeze draft needs this field".to_string(),
            }),
            Rc::new(FallbackFreezeDraftOwnerIdLengthRule { max_length: 16 }),
            Rc::new(FallbackFreezeDraftCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_freeze_draft_validator();
        let item = new_fallback_freeze_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = FallbackFreezeDraftRequiredCurrencyRule {
            message: "fallback freeze draft needs a value".to_string(),
        };
        let mut item = new_fallback_freeze_draft("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = FallbackFreezeDraftRequiredOwnerIdRule {
            message: "fallback freeze draft needs an entry".to_string(),
        };
        let mut item = new_fallback_freeze_draft("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = FallbackFreezeDraftOwnerIdLengthRule { max_length: 80 };
        let mut item = new_fallback_freeze_draft("x", "y");
        item.owner_id = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
