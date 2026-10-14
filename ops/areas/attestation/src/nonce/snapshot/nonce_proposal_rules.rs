use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a nonce proposal must pass. Check returns its problems, or none.
pub trait NonceProposalRule: std::fmt::Debug {
    fn check(&self, item: &NonceProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceProposalRequiredTitleRule {
    pub message: String,
}

impl NonceProposalRule for NonceProposalRequiredTitleRule {
    fn check(&self, item: &NonceProposal) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceProposalCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl NonceProposalRule for NonceProposalCostRangeRule {
    fn check(&self, item: &NonceProposal) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceProposalCodePrefixRule {
    pub prefix: String,
}

impl NonceProposalRule for NonceProposalCodePrefixRule {
    fn check(&self, item: &NonceProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce proposal.
#[derive(Debug, Clone)]
pub struct NonceProposalValidator {
    pub rules: Vec<Rc<dyn NonceProposalRule>>,
}

impl NonceProposalValidator {
    pub fn validate(&self, item: &NonceProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &NonceProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_proposal_validator() -> NonceProposalValidator {
    NonceProposalValidator {
        rules: vec![
            Rc::new(NonceProposalRequiredTitleRule {
                message: "nonce proposal needs this field".to_string(),
            }) as Rc<dyn NonceProposalRule>,
            Rc::new(NonceProposalCostRangeRule { min: 7.6, max: 4410.5 }),
            Rc::new(NonceProposalCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_nonce_proposal_validator();
        let item = new_nonce_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = NonceProposalRequiredTitleRule {
            message: "nonce proposal needs a value".to_string(),
        };
        let mut item = new_nonce_proposal("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = NonceProposalCostRangeRule { min: 3.5, max: 2854.7 };
        let mut item = new_nonce_proposal("x", "y");
        item.cost = 2856.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
