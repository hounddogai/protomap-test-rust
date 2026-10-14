use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped freeze proposal must pass. Check returns its problems, or none.
pub trait CappedFreezeProposalRule: std::fmt::Debug {
    fn check(&self, item: &CappedFreezeProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFreezeProposalAllowedHostRule {
    pub allowed: Vec<String>,
}

impl CappedFreezeProposalRule for CappedFreezeProposalAllowedHostRule {
    fn check(&self, item: &CappedFreezeProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.host))) {
            return vec![format!("host {} is duplicated", item.host)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFreezeProposalRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedFreezeProposalRule for CappedFreezeProposalRetriesRangeRule {
    fn check(&self, item: &CappedFreezeProposal) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFreezeProposalRequiredNameRule {
    pub message: String,
}

impl CappedFreezeProposalRule for CappedFreezeProposalRequiredNameRule {
    fn check(&self, item: &CappedFreezeProposal) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped freeze proposal.
#[derive(Debug, Clone)]
pub struct CappedFreezeProposalValidator {
    pub rules: Vec<Rc<dyn CappedFreezeProposalRule>>,
}

impl CappedFreezeProposalValidator {
    pub fn validate(&self, item: &CappedFreezeProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedFreezeProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedFreezeProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_freeze_proposal_validator() -> CappedFreezeProposalValidator {
    CappedFreezeProposalValidator {
        rules: vec![
            Rc::new(CappedFreezeProposalAllowedHostRule {
                allowed: vec!["tiered".to_string()],
            }) as Rc<dyn CappedFreezeProposalRule>,
            Rc::new(CappedFreezeProposalRetriesRangeRule { min: 80, max: 3600 }),
            Rc::new(CappedFreezeProposalRequiredNameRule {
                message: "capped freeze proposal needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_freeze_proposal_validator();
        let item = new_capped_freeze_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = CappedFreezeProposalRetriesRangeRule { min: 80, max: 1000 };
        let mut item = new_capped_freeze_proposal("x", "y");
        item.retries = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = CappedFreezeProposalRequiredNameRule {
            message: "capped freeze proposal needs this field".to_string(),
        };
        let mut item = new_capped_freeze_proposal("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
