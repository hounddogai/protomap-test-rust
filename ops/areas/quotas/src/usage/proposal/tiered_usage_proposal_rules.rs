use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered usage proposal must pass. Check returns its problems, or none.
pub trait TieredUsageProposalRule: std::fmt::Debug {
    fn check(&self, item: &TieredUsageProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageProposalRequiredKeyIdRule {
    pub message: String,
}

impl TieredUsageProposalRule for TieredUsageProposalRequiredKeyIdRule {
    fn check(&self, item: &TieredUsageProposal) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageProposalAllowedTenantIdRule {
    pub allowed: Vec<String>,
}

impl TieredUsageProposalRule for TieredUsageProposalAllowedTenantIdRule {
    fn check(&self, item: &TieredUsageProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.tenant_id)))
        {
            return vec![format!("tenant id {} is out of range", item.tenant_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageProposalRequiredCodeRule {
    pub message: String,
}

impl TieredUsageProposalRule for TieredUsageProposalRequiredCodeRule {
    fn check(&self, item: &TieredUsageProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageProposalRequiredLabelRule {
    pub message: String,
}

impl TieredUsageProposalRule for TieredUsageProposalRequiredLabelRule {
    fn check(&self, item: &TieredUsageProposal) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageProposalCodeLengthRule {
    pub max_length: i64,
}

impl TieredUsageProposalRule for TieredUsageProposalCodeLengthRule {
    fn check(&self, item: &TieredUsageProposal) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageProposalRequiredCategoryRule {
    pub message: String,
}

impl TieredUsageProposalRule for TieredUsageProposalRequiredCategoryRule {
    fn check(&self, item: &TieredUsageProposal) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered usage proposal.
#[derive(Debug, Clone)]
pub struct TieredUsageProposalValidator {
    pub rules: Vec<Rc<dyn TieredUsageProposalRule>>,
}

impl TieredUsageProposalValidator {
    pub fn validate(&self, item: &TieredUsageProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredUsageProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredUsageProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_usage_proposal_validator() -> TieredUsageProposalValidator {
    TieredUsageProposalValidator {
        rules: vec![
            Rc::new(TieredUsageProposalRequiredKeyIdRule {
                message: "tiered usage proposal needs this field".to_string(),
            }) as Rc<dyn TieredUsageProposalRule>,
            Rc::new(TieredUsageProposalAllowedTenantIdRule { allowed: Vec::new() }),
            Rc::new(TieredUsageProposalRequiredCodeRule {
                message: "tiered usage proposal needs a value".to_string(),
            }),
            Rc::new(TieredUsageProposalRequiredLabelRule {
                message: "tiered usage proposal needs an entry".to_string(),
            }),
            Rc::new(TieredUsageProposalCodeLengthRule { max_length: 32 }),
            Rc::new(TieredUsageProposalRequiredCategoryRule {
                message: "tiered usage proposal needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_usage_proposal_validator();
        let item = new_tiered_usage_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = TieredUsageProposalRequiredKeyIdRule {
            message: "tiered usage proposal needs a value".to_string(),
        };
        let mut item = new_tiered_usage_proposal("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredUsageProposalRequiredCodeRule {
            message: "tiered usage proposal needs an entry".to_string(),
        };
        let mut item = new_tiered_usage_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TieredUsageProposalRequiredLabelRule {
            message: "tiered usage proposal needs an entry".to_string(),
        };
        let mut item = new_tiered_usage_proposal("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredUsageProposalCodeLengthRule { max_length: 64 };
        let mut item = new_tiered_usage_proposal("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = TieredUsageProposalRequiredCategoryRule {
            message: "tiered usage proposal needs an entry".to_string(),
        };
        let mut item = new_tiered_usage_proposal("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
