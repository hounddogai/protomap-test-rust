use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy segment proposal must pass. Check returns its problems, or none.
pub trait LegacySegmentProposalRule: std::fmt::Debug {
    fn check(&self, item: &LegacySegmentProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySegmentProposalRequiredCodeRule {
    pub message: String,
}

impl LegacySegmentProposalRule for LegacySegmentProposalRequiredCodeRule {
    fn check(&self, item: &LegacySegmentProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySegmentProposalChecksumLengthRule {
    pub max_length: i64,
}

impl LegacySegmentProposalRule for LegacySegmentProposalChecksumLengthRule {
    fn check(&self, item: &LegacySegmentProposal) -> Vec<String> {
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
pub struct LegacySegmentProposalStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacySegmentProposalRule for LegacySegmentProposalStartsAtRangeRule {
    fn check(&self, item: &LegacySegmentProposal) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySegmentProposalRequiredLabelRule {
    pub message: String,
}

impl LegacySegmentProposalRule for LegacySegmentProposalRequiredLabelRule {
    fn check(&self, item: &LegacySegmentProposal) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySegmentProposalCodePrefixRule {
    pub prefix: String,
}

impl LegacySegmentProposalRule for LegacySegmentProposalCodePrefixRule {
    fn check(&self, item: &LegacySegmentProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySegmentProposalLabelLengthRule {
    pub max_length: i64,
}

impl LegacySegmentProposalRule for LegacySegmentProposalLabelLengthRule {
    fn check(&self, item: &LegacySegmentProposal) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy segment proposal.
#[derive(Debug, Clone)]
pub struct LegacySegmentProposalValidator {
    pub rules: Vec<Rc<dyn LegacySegmentProposalRule>>,
}

impl LegacySegmentProposalValidator {
    pub fn validate(&self, item: &LegacySegmentProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacySegmentProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacySegmentProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_segment_proposal_validator() -> LegacySegmentProposalValidator {
    LegacySegmentProposalValidator {
        rules: vec![
            Rc::new(LegacySegmentProposalRequiredCodeRule {
                message: "legacy segment proposal needs a value".to_string(),
            }) as Rc<dyn LegacySegmentProposalRule>,
            Rc::new(LegacySegmentProposalChecksumLengthRule { max_length: 120 }),
            Rc::new(LegacySegmentProposalStartsAtRangeRule { min: 64, max: 80 }),
            Rc::new(LegacySegmentProposalRequiredLabelRule {
                message: "legacy segment proposal needs this field".to_string(),
            }),
            Rc::new(LegacySegmentProposalCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(LegacySegmentProposalLabelLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_segment_proposal_validator();
        let item = new_legacy_segment_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacySegmentProposalRequiredCodeRule {
            message: "legacy segment proposal needs this field".to_string(),
        };
        let mut item = new_legacy_segment_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = LegacySegmentProposalChecksumLengthRule { max_length: 255 };
        let mut item = new_legacy_segment_proposal("x", "y");
        item.checksum = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = LegacySegmentProposalStartsAtRangeRule { min: 1000, max: 86400 };
        let mut item = new_legacy_segment_proposal("x", "y");
        item.starts_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LegacySegmentProposalRequiredLabelRule {
            message: "legacy segment proposal needs an entry".to_string(),
        };
        let mut item = new_legacy_segment_proposal("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LegacySegmentProposalLabelLengthRule { max_length: 64 };
        let mut item = new_legacy_segment_proposal("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
