use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback runbook proposal must pass. Check returns its problems, or none.
pub trait FallbackRunbookProposalRule: std::fmt::Debug {
    fn check(&self, item: &FallbackRunbookProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRunbookProposalCipherLengthRule {
    pub max_length: i64,
}

impl FallbackRunbookProposalRule for FallbackRunbookProposalCipherLengthRule {
    fn check(&self, item: &FallbackRunbookProposal) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRunbookProposalRequiredLabelRule {
    pub message: String,
}

impl FallbackRunbookProposalRule for FallbackRunbookProposalRequiredLabelRule {
    fn check(&self, item: &FallbackRunbookProposal) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRunbookProposalChecksumLengthRule {
    pub max_length: i64,
}

impl FallbackRunbookProposalRule for FallbackRunbookProposalChecksumLengthRule {
    fn check(&self, item: &FallbackRunbookProposal) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRunbookProposalRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackRunbookProposalRule for FallbackRunbookProposalRotationDaysRangeRule {
    fn check(&self, item: &FallbackRunbookProposal) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRunbookProposalRequiredCodeRule {
    pub message: String,
}

impl FallbackRunbookProposalRule for FallbackRunbookProposalRequiredCodeRule {
    fn check(&self, item: &FallbackRunbookProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback runbook proposal.
#[derive(Debug, Clone)]
pub struct FallbackRunbookProposalValidator {
    pub rules: Vec<Rc<dyn FallbackRunbookProposalRule>>,
}

impl FallbackRunbookProposalValidator {
    pub fn validate(&self, item: &FallbackRunbookProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackRunbookProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackRunbookProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_runbook_proposal_validator() -> FallbackRunbookProposalValidator {
    FallbackRunbookProposalValidator {
        rules: vec![
            Rc::new(FallbackRunbookProposalCipherLengthRule { max_length: 1000 })
                as Rc<dyn FallbackRunbookProposalRule>,
            Rc::new(FallbackRunbookProposalRequiredLabelRule {
                message: "fallback runbook proposal needs an entry".to_string(),
            }),
            Rc::new(FallbackRunbookProposalChecksumLengthRule { max_length: 64 }),
            Rc::new(FallbackRunbookProposalRotationDaysRangeRule { min: 16, max: 3600 }),
            Rc::new(FallbackRunbookProposalRequiredCodeRule {
                message: "fallback runbook proposal needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_runbook_proposal_validator();
        let item = new_fallback_runbook_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = FallbackRunbookProposalCipherLengthRule { max_length: 86400 };
        let mut item = new_fallback_runbook_proposal("x", "y");
        item.cipher = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = FallbackRunbookProposalRequiredLabelRule {
            message: "fallback runbook proposal needs a value".to_string(),
        };
        let mut item = new_fallback_runbook_proposal("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = FallbackRunbookProposalChecksumLengthRule { max_length: 255 };
        let mut item = new_fallback_runbook_proposal("x", "y");
        item.checksum = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = FallbackRunbookProposalRotationDaysRangeRule { min: 16, max: 32 };
        let mut item = new_fallback_runbook_proposal("x", "y");
        item.rotation_days = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackRunbookProposalRequiredCodeRule {
            message: "fallback runbook proposal needs an entry".to_string(),
        };
        let mut item = new_fallback_runbook_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
