use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast proposal must pass. Check returns its problems, or none.
pub trait ForecastProposalRule: std::fmt::Debug {
    fn check(&self, item: &ForecastProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastProposalRequiredAudienceRule {
    pub message: String,
}

impl ForecastProposalRule for ForecastProposalRequiredAudienceRule {
    fn check(&self, item: &ForecastProposal) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastProposalRequiredCurrencyRule {
    pub message: String,
}

impl ForecastProposalRule for ForecastProposalRequiredCurrencyRule {
    fn check(&self, item: &ForecastProposal) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastProposalRequiredCodeRule {
    pub message: String,
}

impl ForecastProposalRule for ForecastProposalRequiredCodeRule {
    fn check(&self, item: &ForecastProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast proposal.
#[derive(Debug, Clone)]
pub struct ForecastProposalValidator {
    pub rules: Vec<Rc<dyn ForecastProposalRule>>,
}

impl ForecastProposalValidator {
    pub fn validate(&self, item: &ForecastProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ForecastProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_proposal_validator() -> ForecastProposalValidator {
    ForecastProposalValidator {
        rules: vec![
            Rc::new(ForecastProposalRequiredAudienceRule {
                message: "forecast proposal needs an entry".to_string(),
            }) as Rc<dyn ForecastProposalRule>,
            Rc::new(ForecastProposalRequiredCurrencyRule {
                message: "forecast proposal needs an entry".to_string(),
            }),
            Rc::new(ForecastProposalRequiredCodeRule {
                message: "forecast proposal needs a value".to_string(),
            }),
        ],
    }
}
