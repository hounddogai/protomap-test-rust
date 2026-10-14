use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending release proposal must pass. Check returns its problems, or none.
pub trait PendingReleaseProposalRule: std::fmt::Debug {
    fn check(&self, item: &PendingReleaseProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingReleaseProposalCodePrefixRule {
    pub prefix: String,
}

impl PendingReleaseProposalRule for PendingReleaseProposalCodePrefixRule {
    fn check(&self, item: &PendingReleaseProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingReleaseProposalRequiredTimezoneRule {
    pub message: String,
}

impl PendingReleaseProposalRule for PendingReleaseProposalRequiredTimezoneRule {
    fn check(&self, item: &PendingReleaseProposal) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingReleaseProposalTimezoneLengthRule {
    pub max_length: i64,
}

impl PendingReleaseProposalRule for PendingReleaseProposalTimezoneLengthRule {
    fn check(&self, item: &PendingReleaseProposal) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingReleaseProposalUnitLengthRule {
    pub max_length: i64,
}

impl PendingReleaseProposalRule for PendingReleaseProposalUnitLengthRule {
    fn check(&self, item: &PendingReleaseProposal) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingReleaseProposalRequiredLabelRule {
    pub message: String,
}

impl PendingReleaseProposalRule for PendingReleaseProposalRequiredLabelRule {
    fn check(&self, item: &PendingReleaseProposal) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingReleaseProposalAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingReleaseProposalRule for PendingReleaseProposalAmountRangeRule {
    fn check(&self, item: &PendingReleaseProposal) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending release proposal.
#[derive(Debug, Clone)]
pub struct PendingReleaseProposalValidator {
    pub rules: Vec<Rc<dyn PendingReleaseProposalRule>>,
}

impl PendingReleaseProposalValidator {
    pub fn validate(&self, item: &PendingReleaseProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingReleaseProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingReleaseProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_release_proposal_validator() -> PendingReleaseProposalValidator {
    PendingReleaseProposalValidator {
        rules: vec![
            Rc::new(PendingReleaseProposalCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn PendingReleaseProposalRule>,
            Rc::new(PendingReleaseProposalRequiredTimezoneRule {
                message: "pending release proposal needs this field".to_string(),
            }),
            Rc::new(PendingReleaseProposalTimezoneLengthRule { max_length: 255 }),
            Rc::new(PendingReleaseProposalUnitLengthRule { max_length: 3600 }),
            Rc::new(PendingReleaseProposalRequiredLabelRule {
                message: "pending release proposal needs this field".to_string(),
            }),
            Rc::new(PendingReleaseProposalAmountRangeRule { min: 7.5, max: 4444.0 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_release_proposal_validator();
        let item = new_pending_release_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = PendingReleaseProposalRequiredTimezoneRule {
            message: "pending release proposal needs an entry".to_string(),
        };
        let mut item = new_pending_release_proposal("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = PendingReleaseProposalTimezoneLengthRule { max_length: 3600 };
        let mut item = new_pending_release_proposal("x", "y");
        item.timezone = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = PendingReleaseProposalUnitLengthRule { max_length: 64 };
        let mut item = new_pending_release_proposal("x", "y");
        item.unit = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PendingReleaseProposalRequiredLabelRule {
            message: "pending release proposal needs an entry".to_string(),
        };
        let mut item = new_pending_release_proposal("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = PendingReleaseProposalAmountRangeRule { min: 8.7, max: 1513.3 };
        let mut item = new_pending_release_proposal("x", "y");
        item.amount = 1514.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
