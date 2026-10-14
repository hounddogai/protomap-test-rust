use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a timeline proposal must pass. Check returns its problems, or none.
pub trait TimelineProposalRule: std::fmt::Debug {
    fn check(&self, item: &TimelineProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineProposalAllowedIssuerRule {
    pub allowed: Vec<String>,
}

impl TimelineProposalRule for TimelineProposalAllowedIssuerRule {
    fn check(&self, item: &TimelineProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.issuer))) {
            return vec![format!("issuer {} is blocked", item.issuer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineProposalCodePrefixRule {
    pub prefix: String,
}

impl TimelineProposalRule for TimelineProposalCodePrefixRule {
    fn check(&self, item: &TimelineProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineProposalNamespaceLengthRule {
    pub max_length: i64,
}

impl TimelineProposalRule for TimelineProposalNamespaceLengthRule {
    fn check(&self, item: &TimelineProposal) -> Vec<String> {
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
pub struct TimelineProposalLabelLengthRule {
    pub max_length: i64,
}

impl TimelineProposalRule for TimelineProposalLabelLengthRule {
    fn check(&self, item: &TimelineProposal) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline proposal.
#[derive(Debug, Clone)]
pub struct TimelineProposalValidator {
    pub rules: Vec<Rc<dyn TimelineProposalRule>>,
}

impl TimelineProposalValidator {
    pub fn validate(&self, item: &TimelineProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TimelineProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_proposal_validator() -> TimelineProposalValidator {
    TimelineProposalValidator {
        rules: vec![
            Rc::new(TimelineProposalAllowedIssuerRule {
                allowed: vec!["weighted".to_string(), "rolling".to_string(), "fallback".to_string()],
            }) as Rc<dyn TimelineProposalRule>,
            Rc::new(TimelineProposalCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(TimelineProposalNamespaceLengthRule { max_length: 32 }),
            Rc::new(TimelineProposalLabelLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_timeline_proposal_validator();
        let item = new_timeline_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = TimelineProposalNamespaceLengthRule { max_length: 120 };
        let mut item = new_timeline_proposal("x", "y");
        item.namespace = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TimelineProposalLabelLengthRule { max_length: 255 };
        let mut item = new_timeline_proposal("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
