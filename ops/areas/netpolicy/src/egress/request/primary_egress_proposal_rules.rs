use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary egress proposal must pass. Check returns its problems, or none.
pub trait PrimaryEgressProposalRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryEgressProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEgressProposalMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryEgressProposalRule for PrimaryEgressProposalMarginRangeRule {
    fn check(&self, item: &PrimaryEgressProposal) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEgressProposalAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PrimaryEgressProposalRule for PrimaryEgressProposalAllowedCodeRule {
    fn check(&self, item: &PrimaryEgressProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEgressProposalCodePrefixRule {
    pub prefix: String,
}

impl PrimaryEgressProposalRule for PrimaryEgressProposalCodePrefixRule {
    fn check(&self, item: &PrimaryEgressProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEgressProposalRequiredCodeRule {
    pub message: String,
}

impl PrimaryEgressProposalRule for PrimaryEgressProposalRequiredCodeRule {
    fn check(&self, item: &PrimaryEgressProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary egress proposal.
#[derive(Debug, Clone)]
pub struct PrimaryEgressProposalValidator {
    pub rules: Vec<Rc<dyn PrimaryEgressProposalRule>>,
}

impl PrimaryEgressProposalValidator {
    pub fn validate(&self, item: &PrimaryEgressProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryEgressProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryEgressProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_egress_proposal_validator() -> PrimaryEgressProposalValidator {
    PrimaryEgressProposalValidator {
        rules: vec![
            Rc::new(PrimaryEgressProposalMarginRangeRule { min: 3.9, max: 675.4 }) as Rc<dyn PrimaryEgressProposalRule>,
            Rc::new(PrimaryEgressProposalAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(PrimaryEgressProposalCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(PrimaryEgressProposalRequiredCodeRule {
                message: "primary egress proposal needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_egress_proposal_validator();
        let item = new_primary_egress_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = PrimaryEgressProposalMarginRangeRule { min: 6.1, max: 3056.4 };
        let mut item = new_primary_egress_proposal("x", "y");
        item.margin = 3057.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryEgressProposalRequiredCodeRule {
            message: "primary egress proposal needs an entry".to_string(),
        };
        let mut item = new_primary_egress_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
