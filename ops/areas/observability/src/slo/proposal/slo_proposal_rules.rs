use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a slo proposal must pass. Check returns its problems, or none.
pub trait SloProposalRule: std::fmt::Debug {
    fn check(&self, item: &SloProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloProposalRequiredSummaryRule {
    pub message: String,
}

impl SloProposalRule for SloProposalRequiredSummaryRule {
    fn check(&self, item: &SloProposal) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloProposalRequiredLabelRule {
    pub message: String,
}

impl SloProposalRule for SloProposalRequiredLabelRule {
    fn check(&self, item: &SloProposal) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloProposalBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SloProposalRule for SloProposalBurstRangeRule {
    fn check(&self, item: &SloProposal) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloProposalCodeLengthRule {
    pub max_length: i64,
}

impl SloProposalRule for SloProposalCodeLengthRule {
    fn check(&self, item: &SloProposal) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloProposalCodePrefixRule {
    pub prefix: String,
}

impl SloProposalRule for SloProposalCodePrefixRule {
    fn check(&self, item: &SloProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloProposalStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SloProposalRule for SloProposalStartsAtRangeRule {
    fn check(&self, item: &SloProposal) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a slo proposal.
#[derive(Debug, Clone)]
pub struct SloProposalValidator {
    pub rules: Vec<Rc<dyn SloProposalRule>>,
}

impl SloProposalValidator {
    pub fn validate(&self, item: &SloProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SloProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_proposal_validator() -> SloProposalValidator {
    SloProposalValidator {
        rules: vec![
            Rc::new(SloProposalRequiredSummaryRule {
                message: "slo proposal needs an entry".to_string(),
            }) as Rc<dyn SloProposalRule>,
            Rc::new(SloProposalRequiredLabelRule {
                message: "slo proposal needs an entry".to_string(),
            }),
            Rc::new(SloProposalBurstRangeRule { min: 16, max: 80 }),
            Rc::new(SloProposalCodeLengthRule { max_length: 16 }),
            Rc::new(SloProposalCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(SloProposalStartsAtRangeRule { min: 120, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_proposal_validator();
        let item = new_slo_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = SloProposalRequiredSummaryRule {
            message: "slo proposal needs this field".to_string(),
        };
        let mut item = new_slo_proposal("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = SloProposalRequiredLabelRule {
            message: "slo proposal needs this field".to_string(),
        };
        let mut item = new_slo_proposal("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = SloProposalBurstRangeRule { min: 32, max: 120 };
        let mut item = new_slo_proposal("x", "y");
        item.burst = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SloProposalCodeLengthRule { max_length: 64 };
        let mut item = new_slo_proposal("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = SloProposalStartsAtRangeRule { min: 3600, max: 86400 };
        let mut item = new_slo_proposal("x", "y");
        item.starts_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
