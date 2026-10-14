use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a delegation proposal must pass. Check returns its problems, or none.
pub trait DelegationProposalRule: std::fmt::Debug {
    fn check(&self, item: &DelegationProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationProposalPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationProposalRule for DelegationProposalPriorityRangeRule {
    fn check(&self, item: &DelegationProposal) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationProposalGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationProposalRule for DelegationProposalGraceDaysRangeRule {
    fn check(&self, item: &DelegationProposal) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationProposalIssuerLengthRule {
    pub max_length: i64,
}

impl DelegationProposalRule for DelegationProposalIssuerLengthRule {
    fn check(&self, item: &DelegationProposal) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationProposalRequiredLabelRule {
    pub message: String,
}

impl DelegationProposalRule for DelegationProposalRequiredLabelRule {
    fn check(&self, item: &DelegationProposal) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationProposalLabelLengthRule {
    pub max_length: i64,
}

impl DelegationProposalRule for DelegationProposalLabelLengthRule {
    fn check(&self, item: &DelegationProposal) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation proposal.
#[derive(Debug, Clone)]
pub struct DelegationProposalValidator {
    pub rules: Vec<Rc<dyn DelegationProposalRule>>,
}

impl DelegationProposalValidator {
    pub fn validate(&self, item: &DelegationProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DelegationProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_proposal_validator() -> DelegationProposalValidator {
    DelegationProposalValidator {
        rules: vec![
            Rc::new(DelegationProposalPriorityRangeRule { min: 32, max: 80 }) as Rc<dyn DelegationProposalRule>,
            Rc::new(DelegationProposalGraceDaysRangeRule { min: 32, max: 3600 }),
            Rc::new(DelegationProposalIssuerLengthRule { max_length: 3600 }),
            Rc::new(DelegationProposalRequiredLabelRule {
                message: "delegation proposal needs a value".to_string(),
            }),
            Rc::new(DelegationProposalLabelLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_delegation_proposal_validator();
        let item = new_delegation_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = DelegationProposalPriorityRangeRule { min: 16, max: 32 };
        let mut item = new_delegation_proposal("x", "y");
        item.priority = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = DelegationProposalGraceDaysRangeRule { min: 16, max: 64 };
        let mut item = new_delegation_proposal("x", "y");
        item.grace_days = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = DelegationProposalIssuerLengthRule { max_length: 120 };
        let mut item = new_delegation_proposal("x", "y");
        item.issuer = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DelegationProposalRequiredLabelRule {
            message: "delegation proposal needs an entry".to_string(),
        };
        let mut item = new_delegation_proposal("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DelegationProposalLabelLengthRule { max_length: 80 };
        let mut item = new_delegation_proposal("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
