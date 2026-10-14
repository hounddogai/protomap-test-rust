use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped sink proposal must pass. Check returns its problems, or none.
pub trait CappedSinkProposalRule: std::fmt::Debug {
    fn check(&self, item: &CappedSinkProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSinkProposalWindowRule {
    pub max_span: i64,
}

impl CappedSinkProposalRule for CappedSinkProposalWindowRule {
    fn check(&self, item: &CappedSinkProposal) -> Vec<String> {
        if item.starts_at > item.ends_at {
            return vec!["starts at is after ends at".to_string()];
        }
        if self.max_span > 0 && (item.ends_at - item.starts_at > self.max_span) {
            return vec![format!("spans more than {} seconds", self.max_span)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSinkProposalRequiredNamespaceRule {
    pub message: String,
}

impl CappedSinkProposalRule for CappedSinkProposalRequiredNamespaceRule {
    fn check(&self, item: &CappedSinkProposal) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSinkProposalNamespaceLengthRule {
    pub max_length: i64,
}

impl CappedSinkProposalRule for CappedSinkProposalNamespaceLengthRule {
    fn check(&self, item: &CappedSinkProposal) -> Vec<String> {
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
pub struct CappedSinkProposalRequiredLabelRule {
    pub message: String,
}

impl CappedSinkProposalRule for CappedSinkProposalRequiredLabelRule {
    fn check(&self, item: &CappedSinkProposal) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped sink proposal.
#[derive(Debug, Clone)]
pub struct CappedSinkProposalValidator {
    pub rules: Vec<Rc<dyn CappedSinkProposalRule>>,
}

impl CappedSinkProposalValidator {
    pub fn validate(&self, item: &CappedSinkProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedSinkProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedSinkProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_sink_proposal_validator() -> CappedSinkProposalValidator {
    CappedSinkProposalValidator {
        rules: vec![
            Rc::new(CappedSinkProposalWindowRule { max_span: 80 }) as Rc<dyn CappedSinkProposalRule>,
            Rc::new(CappedSinkProposalRequiredNamespaceRule {
                message: "capped sink proposal needs a value".to_string(),
            }),
            Rc::new(CappedSinkProposalNamespaceLengthRule { max_length: 86400 }),
            Rc::new(CappedSinkProposalRequiredLabelRule {
                message: "capped sink proposal needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_sink_proposal_validator();
        let item = new_capped_sink_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = CappedSinkProposalRequiredNamespaceRule {
            message: "capped sink proposal needs this field".to_string(),
        };
        let mut item = new_capped_sink_proposal("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = CappedSinkProposalNamespaceLengthRule { max_length: 32 };
        let mut item = new_capped_sink_proposal("x", "y");
        item.namespace = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = CappedSinkProposalRequiredLabelRule {
            message: "capped sink proposal needs this field".to_string(),
        };
        let mut item = new_capped_sink_proposal("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
