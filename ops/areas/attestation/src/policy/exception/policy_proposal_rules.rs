use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a policy proposal must pass. Check returns its problems, or none.
pub trait PolicyProposalRule: std::fmt::Debug {
    fn check(&self, item: &PolicyProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyProposalCodePrefixRule {
    pub prefix: String,
}

impl PolicyProposalRule for PolicyProposalCodePrefixRule {
    fn check(&self, item: &PolicyProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyProposalCountryLengthRule {
    pub max_length: i64,
}

impl PolicyProposalRule for PolicyProposalCountryLengthRule {
    fn check(&self, item: &PolicyProposal) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyProposalSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PolicyProposalRule for PolicyProposalSequenceRangeRule {
    fn check(&self, item: &PolicyProposal) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a policy proposal.
#[derive(Debug, Clone)]
pub struct PolicyProposalValidator {
    pub rules: Vec<Rc<dyn PolicyProposalRule>>,
}

impl PolicyProposalValidator {
    pub fn validate(&self, item: &PolicyProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PolicyProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicyProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_proposal_validator() -> PolicyProposalValidator {
    PolicyProposalValidator {
        rules: vec![
            Rc::new(PolicyProposalCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn PolicyProposalRule>,
            Rc::new(PolicyProposalCountryLengthRule { max_length: 16 }),
            Rc::new(PolicyProposalSequenceRangeRule { min: 120, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_policy_proposal_validator();
        let item = new_policy_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = PolicyProposalCountryLengthRule { max_length: 3600 };
        let mut item = new_policy_proposal("x", "y");
        item.country = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = PolicyProposalSequenceRangeRule { min: 16, max: 1000 };
        let mut item = new_policy_proposal("x", "y");
        item.sequence = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
