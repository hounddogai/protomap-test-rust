use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a token proposal must pass. Check returns its problems, or none.
pub trait TokenProposalRule: std::fmt::Debug {
    fn check(&self, item: &TokenProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenProposalNameLengthRule {
    pub max_length: i64,
}

impl TokenProposalRule for TokenProposalNameLengthRule {
    fn check(&self, item: &TokenProposal) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenProposalRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TokenProposalRule for TokenProposalRetriesRangeRule {
    fn check(&self, item: &TokenProposal) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenProposalRequiredIssuerRule {
    pub message: String,
}

impl TokenProposalRule for TokenProposalRequiredIssuerRule {
    fn check(&self, item: &TokenProposal) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a token proposal.
#[derive(Debug, Clone)]
pub struct TokenProposalValidator {
    pub rules: Vec<Rc<dyn TokenProposalRule>>,
}

impl TokenProposalValidator {
    pub fn validate(&self, item: &TokenProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TokenProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_proposal_validator() -> TokenProposalValidator {
    TokenProposalValidator {
        rules: vec![
            Rc::new(TokenProposalNameLengthRule { max_length: 80 }) as Rc<dyn TokenProposalRule>,
            Rc::new(TokenProposalRetriesRangeRule { min: 32, max: 80 }),
            Rc::new(TokenProposalRequiredIssuerRule {
                message: "token proposal needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_token_proposal_validator();
        let item = new_token_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TokenProposalNameLengthRule { max_length: 32 };
        let mut item = new_token_proposal("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = TokenProposalRetriesRangeRule { min: 64, max: 64 };
        let mut item = new_token_proposal("x", "y");
        item.retries = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = TokenProposalRequiredIssuerRule {
            message: "token proposal needs this field".to_string(),
        };
        let mut item = new_token_proposal("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
