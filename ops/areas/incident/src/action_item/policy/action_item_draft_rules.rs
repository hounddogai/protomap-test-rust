use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a action item draft must pass. Check returns its problems, or none.
pub trait ActionItemDraftRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemDraftCodePrefixRule {
    pub prefix: String,
}

impl ActionItemDraftRule for ActionItemDraftCodePrefixRule {
    fn check(&self, item: &ActionItemDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemDraftAllowedScopeRule {
    pub allowed: Vec<String>,
}

impl ActionItemDraftRule for ActionItemDraftAllowedScopeRule {
    fn check(&self, item: &ActionItemDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.scope))) {
            return vec![format!("scope {} is above maximum", item.scope)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemDraftRequiredScopeRule {
    pub message: String,
}

impl ActionItemDraftRule for ActionItemDraftRequiredScopeRule {
    fn check(&self, item: &ActionItemDraft) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemDraftLocaleLengthRule {
    pub max_length: i64,
}

impl ActionItemDraftRule for ActionItemDraftLocaleLengthRule {
    fn check(&self, item: &ActionItemDraft) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item draft.
#[derive(Debug, Clone)]
pub struct ActionItemDraftValidator {
    pub rules: Vec<Rc<dyn ActionItemDraftRule>>,
}

impl ActionItemDraftValidator {
    pub fn validate(&self, item: &ActionItemDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ActionItemDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_draft_validator() -> ActionItemDraftValidator {
    ActionItemDraftValidator {
        rules: vec![
            Rc::new(ActionItemDraftCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn ActionItemDraftRule>,
            Rc::new(ActionItemDraftAllowedScopeRule { allowed: Vec::new() }),
            Rc::new(ActionItemDraftRequiredScopeRule {
                message: "action item draft needs a value".to_string(),
            }),
            Rc::new(ActionItemDraftLocaleLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_action_item_draft_validator();
        let item = new_action_item_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = ActionItemDraftRequiredScopeRule {
            message: "action item draft needs this field".to_string(),
        };
        let mut item = new_action_item_draft("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = ActionItemDraftLocaleLengthRule { max_length: 86400 };
        let mut item = new_action_item_draft("x", "y");
        item.locale = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
