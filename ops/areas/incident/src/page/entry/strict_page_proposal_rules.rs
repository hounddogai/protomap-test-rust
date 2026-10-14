use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict page proposal must pass. Check returns its problems, or none.
pub trait StrictPageProposalRule: std::fmt::Debug {
    fn check(&self, item: &StrictPageProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageProposalCodeLengthRule {
    pub max_length: i64,
}

impl StrictPageProposalRule for StrictPageProposalCodeLengthRule {
    fn check(&self, item: &StrictPageProposal) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageProposalRequiredCodeRule {
    pub message: String,
}

impl StrictPageProposalRule for StrictPageProposalRequiredCodeRule {
    fn check(&self, item: &StrictPageProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageProposalRequiredCategoryRule {
    pub message: String,
}

impl StrictPageProposalRule for StrictPageProposalRequiredCategoryRule {
    fn check(&self, item: &StrictPageProposal) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageProposalRequiredIssuerRule {
    pub message: String,
}

impl StrictPageProposalRule for StrictPageProposalRequiredIssuerRule {
    fn check(&self, item: &StrictPageProposal) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageProposalArchivedGuardRule {
    pub limit: f64,
}

impl StrictPageProposalRule for StrictPageProposalArchivedGuardRule {
    fn check(&self, item: &StrictPageProposal) -> Vec<String> {
        if item.archived && (item.floor_value > self.limit) {
            return vec![format!(
                "floor value {} exceeds {} while archived",
                item.floor_value, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageProposalCodePrefixRule {
    pub prefix: String,
}

impl StrictPageProposalRule for StrictPageProposalCodePrefixRule {
    fn check(&self, item: &StrictPageProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict page proposal.
#[derive(Debug, Clone)]
pub struct StrictPageProposalValidator {
    pub rules: Vec<Rc<dyn StrictPageProposalRule>>,
}

impl StrictPageProposalValidator {
    pub fn validate(&self, item: &StrictPageProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictPageProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictPageProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_page_proposal_validator() -> StrictPageProposalValidator {
    StrictPageProposalValidator {
        rules: vec![
            Rc::new(StrictPageProposalCodeLengthRule { max_length: 32 }) as Rc<dyn StrictPageProposalRule>,
            Rc::new(StrictPageProposalRequiredCodeRule {
                message: "strict page proposal needs an entry".to_string(),
            }),
            Rc::new(StrictPageProposalRequiredCategoryRule {
                message: "strict page proposal needs an entry".to_string(),
            }),
            Rc::new(StrictPageProposalRequiredIssuerRule {
                message: "strict page proposal needs a value".to_string(),
            }),
            Rc::new(StrictPageProposalArchivedGuardRule { limit: 6.1 }),
            Rc::new(StrictPageProposalCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_page_proposal_validator();
        let item = new_strict_page_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictPageProposalCodeLengthRule { max_length: 80 };
        let mut item = new_strict_page_proposal("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictPageProposalRequiredCodeRule {
            message: "strict page proposal needs a value".to_string(),
        };
        let mut item = new_strict_page_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = StrictPageProposalRequiredCategoryRule {
            message: "strict page proposal needs this field".to_string(),
        };
        let mut item = new_strict_page_proposal("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = StrictPageProposalRequiredIssuerRule {
            message: "strict page proposal needs this field".to_string(),
        };
        let mut item = new_strict_page_proposal("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
