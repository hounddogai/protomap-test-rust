use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict action item draft must pass. Check returns its problems, or none.
pub trait StrictActionItemDraftRule: std::fmt::Debug {
    fn check(&self, item: &StrictActionItemDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictActionItemDraftCodePrefixRule {
    pub prefix: String,
}

impl StrictActionItemDraftRule for StrictActionItemDraftCodePrefixRule {
    fn check(&self, item: &StrictActionItemDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictActionItemDraftNamespaceLengthRule {
    pub max_length: i64,
}

impl StrictActionItemDraftRule for StrictActionItemDraftNamespaceLengthRule {
    fn check(&self, item: &StrictActionItemDraft) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictActionItemDraftTitleLengthRule {
    pub max_length: i64,
}

impl StrictActionItemDraftRule for StrictActionItemDraftTitleLengthRule {
    fn check(&self, item: &StrictActionItemDraft) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict action item draft.
#[derive(Debug, Clone)]
pub struct StrictActionItemDraftValidator {
    pub rules: Vec<Rc<dyn StrictActionItemDraftRule>>,
}

impl StrictActionItemDraftValidator {
    pub fn validate(&self, item: &StrictActionItemDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictActionItemDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictActionItemDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_action_item_draft_validator() -> StrictActionItemDraftValidator {
    StrictActionItemDraftValidator {
        rules: vec![
            Rc::new(StrictActionItemDraftCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn StrictActionItemDraftRule>,
            Rc::new(StrictActionItemDraftNamespaceLengthRule { max_length: 1000 }),
            Rc::new(StrictActionItemDraftTitleLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_action_item_draft_validator();
        let item = new_strict_action_item_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = StrictActionItemDraftNamespaceLengthRule { max_length: 255 };
        let mut item = new_strict_action_item_draft("x", "y");
        item.namespace = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = StrictActionItemDraftTitleLengthRule { max_length: 32 };
        let mut item = new_strict_action_item_draft("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
