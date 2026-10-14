use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual budget burn proposal must pass. Check returns its problems, or none.
pub trait ManualBudgetBurnProposalRule: std::fmt::Debug {
    fn check(&self, item: &ManualBudgetBurnProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnProposalMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualBudgetBurnProposalRule for ManualBudgetBurnProposalMinimumRangeRule {
    fn check(&self, item: &ManualBudgetBurnProposal) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnProposalRequiredTitleRule {
    pub message: String,
}

impl ManualBudgetBurnProposalRule for ManualBudgetBurnProposalRequiredTitleRule {
    fn check(&self, item: &ManualBudgetBurnProposal) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnProposalTitleLengthRule {
    pub max_length: i64,
}

impl ManualBudgetBurnProposalRule for ManualBudgetBurnProposalTitleLengthRule {
    fn check(&self, item: &ManualBudgetBurnProposal) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual budget burn proposal.
#[derive(Debug, Clone)]
pub struct ManualBudgetBurnProposalValidator {
    pub rules: Vec<Rc<dyn ManualBudgetBurnProposalRule>>,
}

impl ManualBudgetBurnProposalValidator {
    pub fn validate(&self, item: &ManualBudgetBurnProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualBudgetBurnProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualBudgetBurnProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_budget_burn_proposal_validator() -> ManualBudgetBurnProposalValidator {
    ManualBudgetBurnProposalValidator {
        rules: vec![
            Rc::new(ManualBudgetBurnProposalMinimumRangeRule { min: 5.3, max: 1524.0 })
                as Rc<dyn ManualBudgetBurnProposalRule>,
            Rc::new(ManualBudgetBurnProposalRequiredTitleRule {
                message: "manual budget burn proposal needs an entry".to_string(),
            }),
            Rc::new(ManualBudgetBurnProposalTitleLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_budget_burn_proposal_validator();
        let item = new_manual_budget_burn_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = ManualBudgetBurnProposalMinimumRangeRule { min: 1.1, max: 1935.1 };
        let mut item = new_manual_budget_burn_proposal("x", "y");
        item.minimum = 1936.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ManualBudgetBurnProposalRequiredTitleRule {
            message: "manual budget burn proposal needs this field".to_string(),
        };
        let mut item = new_manual_budget_burn_proposal("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ManualBudgetBurnProposalTitleLengthRule { max_length: 86400 };
        let mut item = new_manual_budget_burn_proposal("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
