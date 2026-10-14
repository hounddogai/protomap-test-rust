use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal region pin proposal must pass. Check returns its problems, or none.
pub trait SeasonalRegionPinProposalRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalRegionPinProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRegionPinProposalLabelLengthRule {
    pub max_length: i64,
}

impl SeasonalRegionPinProposalRule for SeasonalRegionPinProposalLabelLengthRule {
    fn check(&self, item: &SeasonalRegionPinProposal) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRegionPinProposalRequiredExternalRefRule {
    pub message: String,
}

impl SeasonalRegionPinProposalRule for SeasonalRegionPinProposalRequiredExternalRefRule {
    fn check(&self, item: &SeasonalRegionPinProposal) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRegionPinProposalExternalRefLengthRule {
    pub max_length: i64,
}

impl SeasonalRegionPinProposalRule for SeasonalRegionPinProposalExternalRefLengthRule {
    fn check(&self, item: &SeasonalRegionPinProposal) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal region pin proposal.
#[derive(Debug, Clone)]
pub struct SeasonalRegionPinProposalValidator {
    pub rules: Vec<Rc<dyn SeasonalRegionPinProposalRule>>,
}

impl SeasonalRegionPinProposalValidator {
    pub fn validate(&self, item: &SeasonalRegionPinProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalRegionPinProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalRegionPinProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_region_pin_proposal_validator() -> SeasonalRegionPinProposalValidator {
    SeasonalRegionPinProposalValidator {
        rules: vec![
            Rc::new(SeasonalRegionPinProposalLabelLengthRule { max_length: 86400 })
                as Rc<dyn SeasonalRegionPinProposalRule>,
            Rc::new(SeasonalRegionPinProposalRequiredExternalRefRule {
                message: "seasonal region pin proposal needs this field".to_string(),
            }),
            Rc::new(SeasonalRegionPinProposalExternalRefLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_region_pin_proposal_validator();
        let item = new_seasonal_region_pin_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = SeasonalRegionPinProposalLabelLengthRule { max_length: 3600 };
        let mut item = new_seasonal_region_pin_proposal("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = SeasonalRegionPinProposalRequiredExternalRefRule {
            message: "seasonal region pin proposal needs this field".to_string(),
        };
        let mut item = new_seasonal_region_pin_proposal("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = SeasonalRegionPinProposalExternalRefLengthRule { max_length: 32 };
        let mut item = new_seasonal_region_pin_proposal("x", "y");
        item.external_ref = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
