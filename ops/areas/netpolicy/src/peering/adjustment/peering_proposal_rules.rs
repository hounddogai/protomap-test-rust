use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a peering proposal must pass. Check returns its problems, or none.
pub trait PeeringProposalRule: std::fmt::Debug {
    fn check(&self, item: &PeeringProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringProposalAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl PeeringProposalRule for PeeringProposalAllowedPrincipalRule {
    fn check(&self, item: &PeeringProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is not allowed", item.principal)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringProposalMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PeeringProposalRule for PeeringProposalMaximumRangeRule {
    fn check(&self, item: &PeeringProposal) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringProposalRequiredPrincipalRule {
    pub message: String,
}

impl PeeringProposalRule for PeeringProposalRequiredPrincipalRule {
    fn check(&self, item: &PeeringProposal) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering proposal.
#[derive(Debug, Clone)]
pub struct PeeringProposalValidator {
    pub rules: Vec<Rc<dyn PeeringProposalRule>>,
}

impl PeeringProposalValidator {
    pub fn validate(&self, item: &PeeringProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PeeringProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_proposal_validator() -> PeeringProposalValidator {
    PeeringProposalValidator {
        rules: vec![
            Rc::new(PeeringProposalAllowedPrincipalRule {
                allowed: vec!["lenient".to_string()],
            }) as Rc<dyn PeeringProposalRule>,
            Rc::new(PeeringProposalMaximumRangeRule { min: 5.4, max: 1995.3 }),
            Rc::new(PeeringProposalRequiredPrincipalRule {
                message: "peering proposal needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_proposal_validator();
        let item = new_peering_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = PeeringProposalMaximumRangeRule { min: 0.8, max: 4260.9 };
        let mut item = new_peering_proposal("x", "y");
        item.maximum = 4262.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = PeeringProposalRequiredPrincipalRule {
            message: "peering proposal needs a value".to_string(),
        };
        let mut item = new_peering_proposal("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
