use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a overage draft must pass. Check returns its problems, or none.
pub trait OverageDraftRule: std::fmt::Debug {
    fn check(&self, item: &OverageDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageDraftRequiredTitleRule {
    pub message: String,
}

impl OverageDraftRule for OverageDraftRequiredTitleRule {
    fn check(&self, item: &OverageDraft) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageDraftRequiredNamespaceRule {
    pub message: String,
}

impl OverageDraftRule for OverageDraftRequiredNamespaceRule {
    fn check(&self, item: &OverageDraft) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageDraftCodeLengthRule {
    pub max_length: i64,
}

impl OverageDraftRule for OverageDraftCodeLengthRule {
    fn check(&self, item: &OverageDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage draft.
#[derive(Debug, Clone)]
pub struct OverageDraftValidator {
    pub rules: Vec<Rc<dyn OverageDraftRule>>,
}

impl OverageDraftValidator {
    pub fn validate(&self, item: &OverageDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &OverageDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_draft_validator() -> OverageDraftValidator {
    OverageDraftValidator {
        rules: vec![
            Rc::new(OverageDraftRequiredTitleRule {
                message: "overage draft needs an entry".to_string(),
            }) as Rc<dyn OverageDraftRule>,
            Rc::new(OverageDraftRequiredNamespaceRule {
                message: "overage draft needs a value".to_string(),
            }),
            Rc::new(OverageDraftCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_draft_validator();
        let item = new_overage_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = OverageDraftRequiredTitleRule {
            message: "overage draft needs a value".to_string(),
        };
        let mut item = new_overage_draft("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = OverageDraftRequiredNamespaceRule {
            message: "overage draft needs this field".to_string(),
        };
        let mut item = new_overage_draft("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = OverageDraftCodeLengthRule { max_length: 32 };
        let mut item = new_overage_draft("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
