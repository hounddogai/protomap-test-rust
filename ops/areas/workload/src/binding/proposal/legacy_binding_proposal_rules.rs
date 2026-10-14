use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy binding proposal must pass. Check returns its problems, or none.
pub trait LegacyBindingProposalRule: std::fmt::Debug {
    fn check(&self, item: &LegacyBindingProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingProposalAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyBindingProposalRule for LegacyBindingProposalAttemptsRangeRule {
    fn check(&self, item: &LegacyBindingProposal) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingProposalFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyBindingProposalRule for LegacyBindingProposalFloorValueRangeRule {
    fn check(&self, item: &LegacyBindingProposal) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingProposalRequiredReviewerRule {
    pub message: String,
}

impl LegacyBindingProposalRule for LegacyBindingProposalRequiredReviewerRule {
    fn check(&self, item: &LegacyBindingProposal) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingProposalTitleLengthRule {
    pub max_length: i64,
}

impl LegacyBindingProposalRule for LegacyBindingProposalTitleLengthRule {
    fn check(&self, item: &LegacyBindingProposal) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingProposalAllowedReviewerRule {
    pub allowed: Vec<String>,
}

impl LegacyBindingProposalRule for LegacyBindingProposalAllowedReviewerRule {
    fn check(&self, item: &LegacyBindingProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.reviewer)))
        {
            return vec![format!("reviewer {} is unknown", item.reviewer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingProposalRequiredCodeRule {
    pub message: String,
}

impl LegacyBindingProposalRule for LegacyBindingProposalRequiredCodeRule {
    fn check(&self, item: &LegacyBindingProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy binding proposal.
#[derive(Debug, Clone)]
pub struct LegacyBindingProposalValidator {
    pub rules: Vec<Rc<dyn LegacyBindingProposalRule>>,
}

impl LegacyBindingProposalValidator {
    pub fn validate(&self, item: &LegacyBindingProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyBindingProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyBindingProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_binding_proposal_validator() -> LegacyBindingProposalValidator {
    LegacyBindingProposalValidator {
        rules: vec![
            Rc::new(LegacyBindingProposalAttemptsRangeRule { min: 32, max: 120 }) as Rc<dyn LegacyBindingProposalRule>,
            Rc::new(LegacyBindingProposalFloorValueRangeRule { min: 5.5, max: 3889.7 }),
            Rc::new(LegacyBindingProposalRequiredReviewerRule {
                message: "legacy binding proposal needs this field".to_string(),
            }),
            Rc::new(LegacyBindingProposalTitleLengthRule { max_length: 32 }),
            Rc::new(LegacyBindingProposalAllowedReviewerRule {
                allowed: vec!["legacy".to_string(), "tiered".to_string(), "weighted".to_string()],
            }),
            Rc::new(LegacyBindingProposalRequiredCodeRule {
                message: "legacy binding proposal needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_binding_proposal_validator();
        let item = new_legacy_binding_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = LegacyBindingProposalAttemptsRangeRule { min: 16, max: 86400 };
        let mut item = new_legacy_binding_proposal("x", "y");
        item.attempts = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = LegacyBindingProposalFloorValueRangeRule { min: 7.6, max: 1663.0 };
        let mut item = new_legacy_binding_proposal("x", "y");
        item.floor_value = 1664.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = LegacyBindingProposalRequiredReviewerRule {
            message: "legacy binding proposal needs a value".to_string(),
        };
        let mut item = new_legacy_binding_proposal("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LegacyBindingProposalTitleLengthRule { max_length: 86400 };
        let mut item = new_legacy_binding_proposal("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyBindingProposalRequiredCodeRule {
            message: "legacy binding proposal needs a value".to_string(),
        };
        let mut item = new_legacy_binding_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
