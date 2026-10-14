use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional health check proposal must pass. Check returns its problems, or none.
pub trait RegionalHealthCheckProposalRule: std::fmt::Debug {
    fn check(&self, item: &RegionalHealthCheckProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalHealthCheckProposalVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalHealthCheckProposalRule for RegionalHealthCheckProposalVersionRangeRule {
    fn check(&self, item: &RegionalHealthCheckProposal) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalHealthCheckProposalCodePrefixRule {
    pub prefix: String,
}

impl RegionalHealthCheckProposalRule for RegionalHealthCheckProposalCodePrefixRule {
    fn check(&self, item: &RegionalHealthCheckProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalHealthCheckProposalCodeLengthRule {
    pub max_length: i64,
}

impl RegionalHealthCheckProposalRule for RegionalHealthCheckProposalCodeLengthRule {
    fn check(&self, item: &RegionalHealthCheckProposal) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional health check proposal.
#[derive(Debug, Clone)]
pub struct RegionalHealthCheckProposalValidator {
    pub rules: Vec<Rc<dyn RegionalHealthCheckProposalRule>>,
}

impl RegionalHealthCheckProposalValidator {
    pub fn validate(&self, item: &RegionalHealthCheckProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalHealthCheckProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalHealthCheckProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_health_check_proposal_validator() -> RegionalHealthCheckProposalValidator {
    RegionalHealthCheckProposalValidator {
        rules: vec![
            Rc::new(RegionalHealthCheckProposalVersionRangeRule { min: 64, max: 255 })
                as Rc<dyn RegionalHealthCheckProposalRule>,
            Rc::new(RegionalHealthCheckProposalCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(RegionalHealthCheckProposalCodeLengthRule { max_length: 80 }),
        ],
    }
}
