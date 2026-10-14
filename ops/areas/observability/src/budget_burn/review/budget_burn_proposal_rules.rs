use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a budget burn proposal must pass. Check returns its problems, or none.
pub trait BudgetBurnProposalRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnProposalNameLengthRule {
    pub max_length: i64,
}

impl BudgetBurnProposalRule for BudgetBurnProposalNameLengthRule {
    fn check(&self, item: &BudgetBurnProposal) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnProposalRequiredSourceRule {
    pub message: String,
}

impl BudgetBurnProposalRule for BudgetBurnProposalRequiredSourceRule {
    fn check(&self, item: &BudgetBurnProposal) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnProposalSourceLengthRule {
    pub max_length: i64,
}

impl BudgetBurnProposalRule for BudgetBurnProposalSourceLengthRule {
    fn check(&self, item: &BudgetBurnProposal) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a budget burn proposal.
#[derive(Debug, Clone)]
pub struct BudgetBurnProposalValidator {
    pub rules: Vec<Rc<dyn BudgetBurnProposalRule>>,
}

impl BudgetBurnProposalValidator {
    pub fn validate(&self, item: &BudgetBurnProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BudgetBurnProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_proposal_validator() -> BudgetBurnProposalValidator {
    BudgetBurnProposalValidator {
        rules: vec![
            Rc::new(BudgetBurnProposalNameLengthRule { max_length: 1000 }) as Rc<dyn BudgetBurnProposalRule>,
            Rc::new(BudgetBurnProposalRequiredSourceRule {
                message: "budget burn proposal needs this field".to_string(),
            }),
            Rc::new(BudgetBurnProposalSourceLengthRule { max_length: 1000 }),
        ],
    }
}
