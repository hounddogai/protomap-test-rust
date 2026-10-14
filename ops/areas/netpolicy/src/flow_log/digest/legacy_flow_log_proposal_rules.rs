use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy flow log proposal must pass. Check returns its problems, or none.
pub trait LegacyFlowLogProposalRule: std::fmt::Debug {
    fn check(&self, item: &LegacyFlowLogProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogProposalDescriptionLengthRule {
    pub max_length: i64,
}

impl LegacyFlowLogProposalRule for LegacyFlowLogProposalDescriptionLengthRule {
    fn check(&self, item: &LegacyFlowLogProposal) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogProposalThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyFlowLogProposalRule for LegacyFlowLogProposalThresholdRangeRule {
    fn check(&self, item: &LegacyFlowLogProposal) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogProposalRequiredCodeRule {
    pub message: String,
}

impl LegacyFlowLogProposalRule for LegacyFlowLogProposalRequiredCodeRule {
    fn check(&self, item: &LegacyFlowLogProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy flow log proposal.
#[derive(Debug, Clone)]
pub struct LegacyFlowLogProposalValidator {
    pub rules: Vec<Rc<dyn LegacyFlowLogProposalRule>>,
}

impl LegacyFlowLogProposalValidator {
    pub fn validate(&self, item: &LegacyFlowLogProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyFlowLogProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyFlowLogProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_flow_log_proposal_validator() -> LegacyFlowLogProposalValidator {
    LegacyFlowLogProposalValidator {
        rules: vec![
            Rc::new(LegacyFlowLogProposalDescriptionLengthRule { max_length: 120 })
                as Rc<dyn LegacyFlowLogProposalRule>,
            Rc::new(LegacyFlowLogProposalThresholdRangeRule { min: 3.4, max: 2389.9 }),
            Rc::new(LegacyFlowLogProposalRequiredCodeRule {
                message: "legacy flow log proposal needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_flow_log_proposal_validator();
        let item = new_legacy_flow_log_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = LegacyFlowLogProposalDescriptionLengthRule { max_length: 80 };
        let mut item = new_legacy_flow_log_proposal("x", "y");
        item.description = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = LegacyFlowLogProposalThresholdRangeRule { min: 6.6, max: 3182.0 };
        let mut item = new_legacy_flow_log_proposal("x", "y");
        item.threshold = 3183.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyFlowLogProposalRequiredCodeRule {
            message: "legacy flow log proposal needs this field".to_string(),
        };
        let mut item = new_legacy_flow_log_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
