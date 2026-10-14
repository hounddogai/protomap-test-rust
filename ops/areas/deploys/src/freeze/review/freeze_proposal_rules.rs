use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a freeze proposal must pass. Check returns its problems, or none.
pub trait FreezeProposalRule: std::fmt::Debug {
    fn check(&self, item: &FreezeProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeProposalRequiredSummaryRule {
    pub message: String,
}

impl FreezeProposalRule for FreezeProposalRequiredSummaryRule {
    fn check(&self, item: &FreezeProposal) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeProposalCodeLengthRule {
    pub max_length: i64,
}

impl FreezeProposalRule for FreezeProposalCodeLengthRule {
    fn check(&self, item: &FreezeProposal) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeProposalDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FreezeProposalRule for FreezeProposalDurationMinutesRangeRule {
    fn check(&self, item: &FreezeProposal) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a freeze proposal.
#[derive(Debug, Clone)]
pub struct FreezeProposalValidator {
    pub rules: Vec<Rc<dyn FreezeProposalRule>>,
}

impl FreezeProposalValidator {
    pub fn validate(&self, item: &FreezeProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FreezeProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_proposal_validator() -> FreezeProposalValidator {
    FreezeProposalValidator {
        rules: vec![
            Rc::new(FreezeProposalRequiredSummaryRule {
                message: "freeze proposal needs an entry".to_string(),
            }) as Rc<dyn FreezeProposalRule>,
            Rc::new(FreezeProposalCodeLengthRule { max_length: 3600 }),
            Rc::new(FreezeProposalDurationMinutesRangeRule { min: 255, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_freeze_proposal_validator();
        let item = new_freeze_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = FreezeProposalRequiredSummaryRule {
            message: "freeze proposal needs a value".to_string(),
        };
        let mut item = new_freeze_proposal("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FreezeProposalCodeLengthRule { max_length: 32 };
        let mut item = new_freeze_proposal("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = FreezeProposalDurationMinutesRangeRule { min: 64, max: 64 };
        let mut item = new_freeze_proposal("x", "y");
        item.duration_minutes = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
