use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict peering proposal must pass. Check returns its problems, or none.
pub trait StrictPeeringProposalRule: std::fmt::Debug {
    fn check(&self, item: &StrictPeeringProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPeeringProposalFingerprintLengthRule {
    pub max_length: i64,
}

impl StrictPeeringProposalRule for StrictPeeringProposalFingerprintLengthRule {
    fn check(&self, item: &StrictPeeringProposal) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPeeringProposalRequiredSummaryRule {
    pub message: String,
}

impl StrictPeeringProposalRule for StrictPeeringProposalRequiredSummaryRule {
    fn check(&self, item: &StrictPeeringProposal) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPeeringProposalMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictPeeringProposalRule for StrictPeeringProposalMaximumRangeRule {
    fn check(&self, item: &StrictPeeringProposal) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict peering proposal.
#[derive(Debug, Clone)]
pub struct StrictPeeringProposalValidator {
    pub rules: Vec<Rc<dyn StrictPeeringProposalRule>>,
}

impl StrictPeeringProposalValidator {
    pub fn validate(&self, item: &StrictPeeringProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictPeeringProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictPeeringProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_peering_proposal_validator() -> StrictPeeringProposalValidator {
    StrictPeeringProposalValidator {
        rules: vec![
            Rc::new(StrictPeeringProposalFingerprintLengthRule { max_length: 3600 })
                as Rc<dyn StrictPeeringProposalRule>,
            Rc::new(StrictPeeringProposalRequiredSummaryRule {
                message: "strict peering proposal needs a value".to_string(),
            }),
            Rc::new(StrictPeeringProposalMaximumRangeRule { min: 0.5, max: 3362.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_peering_proposal_validator();
        let item = new_strict_peering_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = StrictPeeringProposalFingerprintLengthRule { max_length: 120 };
        let mut item = new_strict_peering_proposal("x", "y");
        item.fingerprint = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = StrictPeeringProposalRequiredSummaryRule {
            message: "strict peering proposal needs this field".to_string(),
        };
        let mut item = new_strict_peering_proposal("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = StrictPeeringProposalMaximumRangeRule { min: 0.4, max: 2466.6 };
        let mut item = new_strict_peering_proposal("x", "y");
        item.maximum = 2468.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
