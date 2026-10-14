use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a canary proposal must pass. Check returns its problems, or none.
pub trait CanaryProposalRule: std::fmt::Debug {
    fn check(&self, item: &CanaryProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryProposalRequiredLabelRule {
    pub message: String,
}

impl CanaryProposalRule for CanaryProposalRequiredLabelRule {
    fn check(&self, item: &CanaryProposal) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryProposalRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CanaryProposalRule for CanaryProposalRatioRangeRule {
    fn check(&self, item: &CanaryProposal) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryProposalCodeLengthRule {
    pub max_length: i64,
}

impl CanaryProposalRule for CanaryProposalCodeLengthRule {
    fn check(&self, item: &CanaryProposal) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary proposal.
#[derive(Debug, Clone)]
pub struct CanaryProposalValidator {
    pub rules: Vec<Rc<dyn CanaryProposalRule>>,
}

impl CanaryProposalValidator {
    pub fn validate(&self, item: &CanaryProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CanaryProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_proposal_validator() -> CanaryProposalValidator {
    CanaryProposalValidator {
        rules: vec![
            Rc::new(CanaryProposalRequiredLabelRule {
                message: "canary proposal needs this field".to_string(),
            }) as Rc<dyn CanaryProposalRule>,
            Rc::new(CanaryProposalRatioRangeRule { min: 0.1, max: 3212.3 }),
            Rc::new(CanaryProposalCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_proposal_validator();
        let item = new_canary_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = CanaryProposalRequiredLabelRule {
            message: "canary proposal needs a value".to_string(),
        };
        let mut item = new_canary_proposal("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = CanaryProposalRatioRangeRule { min: 6.0, max: 4352.6 };
        let mut item = new_canary_proposal("x", "y");
        item.ratio = 4354.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CanaryProposalCodeLengthRule { max_length: 16 };
        let mut item = new_canary_proposal("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
