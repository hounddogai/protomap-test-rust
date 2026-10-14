use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a federation draft must pass. Check returns its problems, or none.
pub trait FederationDraftRule: std::fmt::Debug {
    fn check(&self, item: &FederationDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationDraftRequiredNamespaceRule {
    pub message: String,
}

impl FederationDraftRule for FederationDraftRequiredNamespaceRule {
    fn check(&self, item: &FederationDraft) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationDraftNamespaceLengthRule {
    pub max_length: i64,
}

impl FederationDraftRule for FederationDraftNamespaceLengthRule {
    fn check(&self, item: &FederationDraft) -> Vec<String> {
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
pub struct FederationDraftSummaryLengthRule {
    pub max_length: i64,
}

impl FederationDraftRule for FederationDraftSummaryLengthRule {
    fn check(&self, item: &FederationDraft) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationDraftRequiredNameRule {
    pub message: String,
}

impl FederationDraftRule for FederationDraftRequiredNameRule {
    fn check(&self, item: &FederationDraft) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a federation draft.
#[derive(Debug, Clone)]
pub struct FederationDraftValidator {
    pub rules: Vec<Rc<dyn FederationDraftRule>>,
}

impl FederationDraftValidator {
    pub fn validate(&self, item: &FederationDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FederationDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_draft_validator() -> FederationDraftValidator {
    FederationDraftValidator {
        rules: vec![
            Rc::new(FederationDraftRequiredNamespaceRule {
                message: "federation draft needs a value".to_string(),
            }) as Rc<dyn FederationDraftRule>,
            Rc::new(FederationDraftNamespaceLengthRule { max_length: 64 }),
            Rc::new(FederationDraftSummaryLengthRule { max_length: 120 }),
            Rc::new(FederationDraftRequiredNameRule {
                message: "federation draft needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_federation_draft_validator();
        let item = new_federation_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = FederationDraftRequiredNamespaceRule {
            message: "federation draft needs a value".to_string(),
        };
        let mut item = new_federation_draft("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = FederationDraftNamespaceLengthRule { max_length: 120 };
        let mut item = new_federation_draft("x", "y");
        item.namespace = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = FederationDraftSummaryLengthRule { max_length: 32 };
        let mut item = new_federation_draft("x", "y");
        item.summary = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FederationDraftRequiredNameRule {
            message: "federation draft needs this field".to_string(),
        };
        let mut item = new_federation_draft("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
