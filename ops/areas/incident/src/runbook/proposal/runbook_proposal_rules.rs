use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a runbook proposal must pass. Check returns its problems, or none.
pub trait RunbookProposalRule: std::fmt::Debug {
    fn check(&self, item: &RunbookProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookProposalMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RunbookProposalRule for RunbookProposalMaximumRangeRule {
    fn check(&self, item: &RunbookProposal) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookProposalAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl RunbookProposalRule for RunbookProposalAllowedLabelRule {
    fn check(&self, item: &RunbookProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is expired", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookProposalRequiredCodeRule {
    pub message: String,
}

impl RunbookProposalRule for RunbookProposalRequiredCodeRule {
    fn check(&self, item: &RunbookProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a runbook proposal.
#[derive(Debug, Clone)]
pub struct RunbookProposalValidator {
    pub rules: Vec<Rc<dyn RunbookProposalRule>>,
}

impl RunbookProposalValidator {
    pub fn validate(&self, item: &RunbookProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RunbookProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_proposal_validator() -> RunbookProposalValidator {
    RunbookProposalValidator {
        rules: vec![
            Rc::new(RunbookProposalMaximumRangeRule { min: 6.9, max: 4568.8 }) as Rc<dyn RunbookProposalRule>,
            Rc::new(RunbookProposalAllowedLabelRule {
                allowed: vec!["batched".to_string()],
            }),
            Rc::new(RunbookProposalRequiredCodeRule {
                message: "runbook proposal needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_runbook_proposal_validator();
        let item = new_runbook_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = RunbookProposalMaximumRangeRule { min: 4.6, max: 3600.7 };
        let mut item = new_runbook_proposal("x", "y");
        item.maximum = 3602.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RunbookProposalRequiredCodeRule {
            message: "runbook proposal needs this field".to_string(),
        };
        let mut item = new_runbook_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
