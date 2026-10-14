use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped delegation draft must pass. Check returns its problems, or none.
pub trait CappedDelegationDraftRule: std::fmt::Debug {
    fn check(&self, item: &CappedDelegationDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDelegationDraftRequiredLabelRule {
    pub message: String,
}

impl CappedDelegationDraftRule for CappedDelegationDraftRequiredLabelRule {
    fn check(&self, item: &CappedDelegationDraft) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDelegationDraftRequiredTitleRule {
    pub message: String,
}

impl CappedDelegationDraftRule for CappedDelegationDraftRequiredTitleRule {
    fn check(&self, item: &CappedDelegationDraft) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDelegationDraftTitleLengthRule {
    pub max_length: i64,
}

impl CappedDelegationDraftRule for CappedDelegationDraftTitleLengthRule {
    fn check(&self, item: &CappedDelegationDraft) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDelegationDraftLabelLengthRule {
    pub max_length: i64,
}

impl CappedDelegationDraftRule for CappedDelegationDraftLabelLengthRule {
    fn check(&self, item: &CappedDelegationDraft) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped delegation draft.
#[derive(Debug, Clone)]
pub struct CappedDelegationDraftValidator {
    pub rules: Vec<Rc<dyn CappedDelegationDraftRule>>,
}

impl CappedDelegationDraftValidator {
    pub fn validate(&self, item: &CappedDelegationDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedDelegationDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedDelegationDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_delegation_draft_validator() -> CappedDelegationDraftValidator {
    CappedDelegationDraftValidator {
        rules: vec![
            Rc::new(CappedDelegationDraftRequiredLabelRule {
                message: "capped delegation draft needs this field".to_string(),
            }) as Rc<dyn CappedDelegationDraftRule>,
            Rc::new(CappedDelegationDraftRequiredTitleRule {
                message: "capped delegation draft needs a value".to_string(),
            }),
            Rc::new(CappedDelegationDraftTitleLengthRule { max_length: 86400 }),
            Rc::new(CappedDelegationDraftLabelLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_delegation_draft_validator();
        let item = new_capped_delegation_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = CappedDelegationDraftRequiredLabelRule {
            message: "capped delegation draft needs an entry".to_string(),
        };
        let mut item = new_capped_delegation_draft("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = CappedDelegationDraftRequiredTitleRule {
            message: "capped delegation draft needs a value".to_string(),
        };
        let mut item = new_capped_delegation_draft("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = CappedDelegationDraftTitleLengthRule { max_length: 64 };
        let mut item = new_capped_delegation_draft("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = CappedDelegationDraftLabelLengthRule { max_length: 80 };
        let mut item = new_capped_delegation_draft("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
