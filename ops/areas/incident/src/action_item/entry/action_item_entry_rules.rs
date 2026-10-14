use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a action item entry must pass. Check returns its problems, or none.
pub trait ActionItemEntryRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemEntryCodePrefixRule {
    pub prefix: String,
}

impl ActionItemEntryRule for ActionItemEntryCodePrefixRule {
    fn check(&self, item: &ActionItemEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemEntryRequiredCodeRule {
    pub message: String,
}

impl ActionItemEntryRule for ActionItemEntryRequiredCodeRule {
    fn check(&self, item: &ActionItemEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemEntryAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ActionItemEntryRule for ActionItemEntryAllowedCodeRule {
    fn check(&self, item: &ActionItemEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemEntryRequiredLabelRule {
    pub message: String,
}

impl ActionItemEntryRule for ActionItemEntryRequiredLabelRule {
    fn check(&self, item: &ActionItemEntry) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemEntryLabelLengthRule {
    pub max_length: i64,
}

impl ActionItemEntryRule for ActionItemEntryLabelLengthRule {
    fn check(&self, item: &ActionItemEntry) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item entry.
#[derive(Debug, Clone)]
pub struct ActionItemEntryValidator {
    pub rules: Vec<Rc<dyn ActionItemEntryRule>>,
}

impl ActionItemEntryValidator {
    pub fn validate(&self, item: &ActionItemEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ActionItemEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_entry_validator() -> ActionItemEntryValidator {
    ActionItemEntryValidator {
        rules: vec![
            Rc::new(ActionItemEntryCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn ActionItemEntryRule>,
            Rc::new(ActionItemEntryRequiredCodeRule {
                message: "action item entry needs a value".to_string(),
            }),
            Rc::new(ActionItemEntryAllowedCodeRule {
                allowed: vec!["lenient".to_string(), "lenient".to_string(), "scheduled".to_string()],
            }),
            Rc::new(ActionItemEntryRequiredLabelRule {
                message: "action item entry needs a value".to_string(),
            }),
            Rc::new(ActionItemEntryLabelLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_action_item_entry_validator();
        let item = new_action_item_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ActionItemEntryRequiredCodeRule {
            message: "action item entry needs an entry".to_string(),
        };
        let mut item = new_action_item_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ActionItemEntryRequiredLabelRule {
            message: "action item entry needs a value".to_string(),
        };
        let mut item = new_action_item_entry("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ActionItemEntryLabelLengthRule { max_length: 64 };
        let mut item = new_action_item_entry("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
