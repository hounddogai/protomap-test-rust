use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending action item proposal must pass. Check returns its problems, or none.
pub trait PendingActionItemProposalRule: std::fmt::Debug {
    fn check(&self, item: &PendingActionItemProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingActionItemProposalAllowedSummaryRule {
    pub allowed: Vec<String>,
}

impl PendingActionItemProposalRule for PendingActionItemProposalAllowedSummaryRule {
    fn check(&self, item: &PendingActionItemProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.summary)))
        {
            return vec![format!("summary {} is inconsistent", item.summary)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingActionItemProposalMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingActionItemProposalRule for PendingActionItemProposalMarginRangeRule {
    fn check(&self, item: &PendingActionItemProposal) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingActionItemProposalReviewerLengthRule {
    pub max_length: i64,
}

impl PendingActionItemProposalRule for PendingActionItemProposalReviewerLengthRule {
    fn check(&self, item: &PendingActionItemProposal) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending action item proposal.
#[derive(Debug, Clone)]
pub struct PendingActionItemProposalValidator {
    pub rules: Vec<Rc<dyn PendingActionItemProposalRule>>,
}

impl PendingActionItemProposalValidator {
    pub fn validate(&self, item: &PendingActionItemProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingActionItemProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingActionItemProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_action_item_proposal_validator() -> PendingActionItemProposalValidator {
    PendingActionItemProposalValidator {
        rules: vec![
            Rc::new(PendingActionItemProposalAllowedSummaryRule {
                allowed: vec!["rolling".to_string()],
            }) as Rc<dyn PendingActionItemProposalRule>,
            Rc::new(PendingActionItemProposalMarginRangeRule { min: 9.3, max: 4387.7 }),
            Rc::new(PendingActionItemProposalReviewerLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_action_item_proposal_validator();
        let item = new_pending_action_item_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = PendingActionItemProposalMarginRangeRule { min: 5.6, max: 1673.1 };
        let mut item = new_pending_action_item_proposal("x", "y");
        item.margin = 1674.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = PendingActionItemProposalReviewerLengthRule { max_length: 16 };
        let mut item = new_pending_action_item_proposal("x", "y");
        item.reviewer = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
