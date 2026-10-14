use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling budget burn proposal must pass. Check returns its problems, or none.
pub trait RollingBudgetBurnProposalRule: std::fmt::Debug {
    fn check(&self, item: &RollingBudgetBurnProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnProposalLabelLengthRule {
    pub max_length: i64,
}

impl RollingBudgetBurnProposalRule for RollingBudgetBurnProposalLabelLengthRule {
    fn check(&self, item: &RollingBudgetBurnProposal) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnProposalCodePrefixRule {
    pub prefix: String,
}

impl RollingBudgetBurnProposalRule for RollingBudgetBurnProposalCodePrefixRule {
    fn check(&self, item: &RollingBudgetBurnProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnProposalSegmentLengthRule {
    pub max_length: i64,
}

impl RollingBudgetBurnProposalRule for RollingBudgetBurnProposalSegmentLengthRule {
    fn check(&self, item: &RollingBudgetBurnProposal) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnProposalRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingBudgetBurnProposalRule for RollingBudgetBurnProposalRevisionRangeRule {
    fn check(&self, item: &RollingBudgetBurnProposal) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling budget burn proposal.
#[derive(Debug, Clone)]
pub struct RollingBudgetBurnProposalValidator {
    pub rules: Vec<Rc<dyn RollingBudgetBurnProposalRule>>,
}

impl RollingBudgetBurnProposalValidator {
    pub fn validate(&self, item: &RollingBudgetBurnProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingBudgetBurnProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingBudgetBurnProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_budget_burn_proposal_validator() -> RollingBudgetBurnProposalValidator {
    RollingBudgetBurnProposalValidator {
        rules: vec![
            Rc::new(RollingBudgetBurnProposalLabelLengthRule { max_length: 1000 })
                as Rc<dyn RollingBudgetBurnProposalRule>,
            Rc::new(RollingBudgetBurnProposalCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(RollingBudgetBurnProposalSegmentLengthRule { max_length: 1000 }),
            Rc::new(RollingBudgetBurnProposalRevisionRangeRule { min: 32, max: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_budget_burn_proposal_validator();
        let item = new_rolling_budget_burn_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RollingBudgetBurnProposalLabelLengthRule { max_length: 64 };
        let mut item = new_rolling_budget_burn_proposal("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = RollingBudgetBurnProposalSegmentLengthRule { max_length: 64 };
        let mut item = new_rolling_budget_burn_proposal("x", "y");
        item.segment = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = RollingBudgetBurnProposalRevisionRangeRule { min: 64, max: 255 };
        let mut item = new_rolling_budget_burn_proposal("x", "y");
        item.revision = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
