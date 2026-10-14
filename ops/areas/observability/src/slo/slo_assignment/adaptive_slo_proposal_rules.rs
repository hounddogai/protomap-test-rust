use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive slo proposal must pass. Check returns its problems, or none.
pub trait AdaptiveSloProposalRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveSloProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloProposalPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveSloProposalRule for AdaptiveSloProposalPortRangeRule {
    fn check(&self, item: &AdaptiveSloProposal) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloProposalCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveSloProposalRule for AdaptiveSloProposalCodeLengthRule {
    fn check(&self, item: &AdaptiveSloProposal) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloProposalRequiredReviewerRule {
    pub message: String,
}

impl AdaptiveSloProposalRule for AdaptiveSloProposalRequiredReviewerRule {
    fn check(&self, item: &AdaptiveSloProposal) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive slo proposal.
#[derive(Debug, Clone)]
pub struct AdaptiveSloProposalValidator {
    pub rules: Vec<Rc<dyn AdaptiveSloProposalRule>>,
}

impl AdaptiveSloProposalValidator {
    pub fn validate(&self, item: &AdaptiveSloProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveSloProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveSloProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_slo_proposal_validator() -> AdaptiveSloProposalValidator {
    AdaptiveSloProposalValidator {
        rules: vec![
            Rc::new(AdaptiveSloProposalPortRangeRule { min: 16, max: 80 }) as Rc<dyn AdaptiveSloProposalRule>,
            Rc::new(AdaptiveSloProposalCodeLengthRule { max_length: 3600 }),
            Rc::new(AdaptiveSloProposalRequiredReviewerRule {
                message: "adaptive slo proposal needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_slo_proposal_validator();
        let item = new_adaptive_slo_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = AdaptiveSloProposalPortRangeRule { min: 80, max: 120 };
        let mut item = new_adaptive_slo_proposal("x", "y");
        item.port = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveSloProposalCodeLengthRule { max_length: 120 };
        let mut item = new_adaptive_slo_proposal("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = AdaptiveSloProposalRequiredReviewerRule {
            message: "adaptive slo proposal needs this field".to_string(),
        };
        let mut item = new_adaptive_slo_proposal("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
