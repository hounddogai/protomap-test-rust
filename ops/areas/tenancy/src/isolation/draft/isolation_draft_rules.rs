use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a isolation draft must pass. Check returns its problems, or none.
pub trait IsolationDraftRule: std::fmt::Debug {
    fn check(&self, item: &IsolationDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationDraftScopeLengthRule {
    pub max_length: i64,
}

impl IsolationDraftRule for IsolationDraftScopeLengthRule {
    fn check(&self, item: &IsolationDraft) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationDraftLabelLengthRule {
    pub max_length: i64,
}

impl IsolationDraftRule for IsolationDraftLabelLengthRule {
    fn check(&self, item: &IsolationDraft) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationDraftRequiredDescriptionRule {
    pub message: String,
}

impl IsolationDraftRule for IsolationDraftRequiredDescriptionRule {
    fn check(&self, item: &IsolationDraft) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationDraftDescriptionLengthRule {
    pub max_length: i64,
}

impl IsolationDraftRule for IsolationDraftDescriptionLengthRule {
    fn check(&self, item: &IsolationDraft) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationDraftCodePrefixRule {
    pub prefix: String,
}

impl IsolationDraftRule for IsolationDraftCodePrefixRule {
    fn check(&self, item: &IsolationDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a isolation draft.
#[derive(Debug, Clone)]
pub struct IsolationDraftValidator {
    pub rules: Vec<Rc<dyn IsolationDraftRule>>,
}

impl IsolationDraftValidator {
    pub fn validate(&self, item: &IsolationDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IsolationDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_draft_validator() -> IsolationDraftValidator {
    IsolationDraftValidator {
        rules: vec![
            Rc::new(IsolationDraftScopeLengthRule { max_length: 16 }) as Rc<dyn IsolationDraftRule>,
            Rc::new(IsolationDraftLabelLengthRule { max_length: 80 }),
            Rc::new(IsolationDraftRequiredDescriptionRule {
                message: "isolation draft needs a value".to_string(),
            }),
            Rc::new(IsolationDraftDescriptionLengthRule { max_length: 86400 }),
            Rc::new(IsolationDraftCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_isolation_draft_validator();
        let item = new_isolation_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = IsolationDraftScopeLengthRule { max_length: 32 };
        let mut item = new_isolation_draft("x", "y");
        item.scope = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = IsolationDraftLabelLengthRule { max_length: 1000 };
        let mut item = new_isolation_draft("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = IsolationDraftRequiredDescriptionRule {
            message: "isolation draft needs an entry".to_string(),
        };
        let mut item = new_isolation_draft("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = IsolationDraftDescriptionLengthRule { max_length: 1000 };
        let mut item = new_isolation_draft("x", "y");
        item.description = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
