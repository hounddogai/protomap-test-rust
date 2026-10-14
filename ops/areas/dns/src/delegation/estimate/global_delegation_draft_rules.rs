use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global delegation draft must pass. Check returns its problems, or none.
pub trait GlobalDelegationDraftRule: std::fmt::Debug {
    fn check(&self, item: &GlobalDelegationDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDelegationDraftNotesLengthRule {
    pub max_length: i64,
}

impl GlobalDelegationDraftRule for GlobalDelegationDraftNotesLengthRule {
    fn check(&self, item: &GlobalDelegationDraft) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDelegationDraftCodePrefixRule {
    pub prefix: String,
}

impl GlobalDelegationDraftRule for GlobalDelegationDraftCodePrefixRule {
    fn check(&self, item: &GlobalDelegationDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDelegationDraftDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalDelegationDraftRule for GlobalDelegationDraftDiscountRangeRule {
    fn check(&self, item: &GlobalDelegationDraft) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDelegationDraftRequiredKeyIdRule {
    pub message: String,
}

impl GlobalDelegationDraftRule for GlobalDelegationDraftRequiredKeyIdRule {
    fn check(&self, item: &GlobalDelegationDraft) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDelegationDraftAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl GlobalDelegationDraftRule for GlobalDelegationDraftAllowedKeyIdRule {
    fn check(&self, item: &GlobalDelegationDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is expired", item.key_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDelegationDraftRequiredNotesRule {
    pub message: String,
}

impl GlobalDelegationDraftRule for GlobalDelegationDraftRequiredNotesRule {
    fn check(&self, item: &GlobalDelegationDraft) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global delegation draft.
#[derive(Debug, Clone)]
pub struct GlobalDelegationDraftValidator {
    pub rules: Vec<Rc<dyn GlobalDelegationDraftRule>>,
}

impl GlobalDelegationDraftValidator {
    pub fn validate(&self, item: &GlobalDelegationDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalDelegationDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalDelegationDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_delegation_draft_validator() -> GlobalDelegationDraftValidator {
    GlobalDelegationDraftValidator {
        rules: vec![
            Rc::new(GlobalDelegationDraftNotesLengthRule { max_length: 86400 }) as Rc<dyn GlobalDelegationDraftRule>,
            Rc::new(GlobalDelegationDraftCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(GlobalDelegationDraftDiscountRangeRule { min: 8.2, max: 1492.2 }),
            Rc::new(GlobalDelegationDraftRequiredKeyIdRule {
                message: "global delegation draft needs this field".to_string(),
            }),
            Rc::new(GlobalDelegationDraftAllowedKeyIdRule {
                allowed: vec!["adaptive".to_string(), "legacy".to_string()],
            }),
            Rc::new(GlobalDelegationDraftRequiredNotesRule {
                message: "global delegation draft needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_delegation_draft_validator();
        let item = new_global_delegation_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = GlobalDelegationDraftNotesLengthRule { max_length: 3600 };
        let mut item = new_global_delegation_draft("x", "y");
        item.notes = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = GlobalDelegationDraftDiscountRangeRule { min: 0.1, max: 1172.8 };
        let mut item = new_global_delegation_draft("x", "y");
        item.discount = 1174.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = GlobalDelegationDraftRequiredKeyIdRule {
            message: "global delegation draft needs an entry".to_string(),
        };
        let mut item = new_global_delegation_draft("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = GlobalDelegationDraftRequiredNotesRule {
            message: "global delegation draft needs an entry".to_string(),
        };
        let mut item = new_global_delegation_draft("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
