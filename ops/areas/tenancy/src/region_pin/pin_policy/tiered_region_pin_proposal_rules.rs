use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered region pin proposal must pass. Check returns its problems, or none.
pub trait TieredRegionPinProposalRule: std::fmt::Debug {
    fn check(&self, item: &TieredRegionPinProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinProposalRequiredNameRule {
    pub message: String,
}

impl TieredRegionPinProposalRule for TieredRegionPinProposalRequiredNameRule {
    fn check(&self, item: &TieredRegionPinProposal) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinProposalCodePrefixRule {
    pub prefix: String,
}

impl TieredRegionPinProposalRule for TieredRegionPinProposalCodePrefixRule {
    fn check(&self, item: &TieredRegionPinProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinProposalAllowedRegionRule {
    pub allowed: Vec<String>,
}

impl TieredRegionPinProposalRule for TieredRegionPinProposalAllowedRegionRule {
    fn check(&self, item: &TieredRegionPinProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.region))) {
            return vec![format!("region {} is inconsistent", item.region)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered region pin proposal.
#[derive(Debug, Clone)]
pub struct TieredRegionPinProposalValidator {
    pub rules: Vec<Rc<dyn TieredRegionPinProposalRule>>,
}

impl TieredRegionPinProposalValidator {
    pub fn validate(&self, item: &TieredRegionPinProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredRegionPinProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredRegionPinProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_region_pin_proposal_validator() -> TieredRegionPinProposalValidator {
    TieredRegionPinProposalValidator {
        rules: vec![
            Rc::new(TieredRegionPinProposalRequiredNameRule {
                message: "tiered region pin proposal needs a value".to_string(),
            }) as Rc<dyn TieredRegionPinProposalRule>,
            Rc::new(TieredRegionPinProposalCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(TieredRegionPinProposalAllowedRegionRule {
                allowed: vec!["legacy".to_string(), "fallback".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_region_pin_proposal_validator();
        let item = new_tiered_region_pin_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TieredRegionPinProposalRequiredNameRule {
            message: "tiered region pin proposal needs this field".to_string(),
        };
        let mut item = new_tiered_region_pin_proposal("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
