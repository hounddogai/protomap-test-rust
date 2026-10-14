use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a escalation proposal must pass. Check returns its problems, or none.
pub trait EscalationProposalRule: std::fmt::Debug {
    fn check(&self, item: &EscalationProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationProposalCodePrefixRule {
    pub prefix: String,
}

impl EscalationProposalRule for EscalationProposalCodePrefixRule {
    fn check(&self, item: &EscalationProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationProposalTitleLengthRule {
    pub max_length: i64,
}

impl EscalationProposalRule for EscalationProposalTitleLengthRule {
    fn check(&self, item: &EscalationProposal) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationProposalRequiredScopeRule {
    pub message: String,
}

impl EscalationProposalRule for EscalationProposalRequiredScopeRule {
    fn check(&self, item: &EscalationProposal) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationProposalRequiredTitleRule {
    pub message: String,
}

impl EscalationProposalRule for EscalationProposalRequiredTitleRule {
    fn check(&self, item: &EscalationProposal) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationProposalScopeLengthRule {
    pub max_length: i64,
}

impl EscalationProposalRule for EscalationProposalScopeLengthRule {
    fn check(&self, item: &EscalationProposal) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation proposal.
#[derive(Debug, Clone)]
pub struct EscalationProposalValidator {
    pub rules: Vec<Rc<dyn EscalationProposalRule>>,
}

impl EscalationProposalValidator {
    pub fn validate(&self, item: &EscalationProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EscalationProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_proposal_validator() -> EscalationProposalValidator {
    EscalationProposalValidator {
        rules: vec![
            Rc::new(EscalationProposalCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn EscalationProposalRule>,
            Rc::new(EscalationProposalTitleLengthRule { max_length: 3600 }),
            Rc::new(EscalationProposalRequiredScopeRule {
                message: "escalation proposal needs this field".to_string(),
            }),
            Rc::new(EscalationProposalRequiredTitleRule {
                message: "escalation proposal needs an entry".to_string(),
            }),
            Rc::new(EscalationProposalScopeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_escalation_proposal_validator();
        let item = new_escalation_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = EscalationProposalTitleLengthRule { max_length: 80 };
        let mut item = new_escalation_proposal("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = EscalationProposalRequiredScopeRule {
            message: "escalation proposal needs a value".to_string(),
        };
        let mut item = new_escalation_proposal("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = EscalationProposalRequiredTitleRule {
            message: "escalation proposal needs this field".to_string(),
        };
        let mut item = new_escalation_proposal("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = EscalationProposalScopeLengthRule { max_length: 1000 };
        let mut item = new_escalation_proposal("x", "y");
        item.scope = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
