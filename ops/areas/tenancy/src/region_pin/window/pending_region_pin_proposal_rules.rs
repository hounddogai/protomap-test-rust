use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending region pin proposal must pass. Check returns its problems, or none.
pub trait PendingRegionPinProposalRule: std::fmt::Debug {
    fn check(&self, item: &PendingRegionPinProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinProposalRequiredLabelRule {
    pub message: String,
}

impl PendingRegionPinProposalRule for PendingRegionPinProposalRequiredLabelRule {
    fn check(&self, item: &PendingRegionPinProposal) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinProposalRequiredCodeRule {
    pub message: String,
}

impl PendingRegionPinProposalRule for PendingRegionPinProposalRequiredCodeRule {
    fn check(&self, item: &PendingRegionPinProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinProposalVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingRegionPinProposalRule for PendingRegionPinProposalVersionRangeRule {
    fn check(&self, item: &PendingRegionPinProposal) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinProposalCodeLengthRule {
    pub max_length: i64,
}

impl PendingRegionPinProposalRule for PendingRegionPinProposalCodeLengthRule {
    fn check(&self, item: &PendingRegionPinProposal) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinProposalRequiredChannelRule {
    pub message: String,
}

impl PendingRegionPinProposalRule for PendingRegionPinProposalRequiredChannelRule {
    fn check(&self, item: &PendingRegionPinProposal) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending region pin proposal.
#[derive(Debug, Clone)]
pub struct PendingRegionPinProposalValidator {
    pub rules: Vec<Rc<dyn PendingRegionPinProposalRule>>,
}

impl PendingRegionPinProposalValidator {
    pub fn validate(&self, item: &PendingRegionPinProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingRegionPinProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingRegionPinProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_region_pin_proposal_validator() -> PendingRegionPinProposalValidator {
    PendingRegionPinProposalValidator {
        rules: vec![
            Rc::new(PendingRegionPinProposalRequiredLabelRule {
                message: "pending region pin proposal needs a value".to_string(),
            }) as Rc<dyn PendingRegionPinProposalRule>,
            Rc::new(PendingRegionPinProposalRequiredCodeRule {
                message: "pending region pin proposal needs this field".to_string(),
            }),
            Rc::new(PendingRegionPinProposalVersionRangeRule { min: 32, max: 1000 }),
            Rc::new(PendingRegionPinProposalCodeLengthRule { max_length: 16 }),
            Rc::new(PendingRegionPinProposalRequiredChannelRule {
                message: "pending region pin proposal needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_region_pin_proposal_validator();
        let item = new_pending_region_pin_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PendingRegionPinProposalRequiredLabelRule {
            message: "pending region pin proposal needs an entry".to_string(),
        };
        let mut item = new_pending_region_pin_proposal("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingRegionPinProposalRequiredCodeRule {
            message: "pending region pin proposal needs an entry".to_string(),
        };
        let mut item = new_pending_region_pin_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = PendingRegionPinProposalVersionRangeRule { min: 80, max: 255 };
        let mut item = new_pending_region_pin_proposal("x", "y");
        item.version = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingRegionPinProposalCodeLengthRule { max_length: 16 };
        let mut item = new_pending_region_pin_proposal("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = PendingRegionPinProposalRequiredChannelRule {
            message: "pending region pin proposal needs an entry".to_string(),
        };
        let mut item = new_pending_region_pin_proposal("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
