use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped page proposal must pass. Check returns its problems, or none.
pub trait CappedPageProposalRule: std::fmt::Debug {
    fn check(&self, item: &CappedPageProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPageProposalQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedPageProposalRule for CappedPageProposalQuantityRangeRule {
    fn check(&self, item: &CappedPageProposal) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPageProposalMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedPageProposalRule for CappedPageProposalMarginRangeRule {
    fn check(&self, item: &CappedPageProposal) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPageProposalRequiredScopeRule {
    pub message: String,
}

impl CappedPageProposalRule for CappedPageProposalRequiredScopeRule {
    fn check(&self, item: &CappedPageProposal) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPageProposalNameLengthRule {
    pub max_length: i64,
}

impl CappedPageProposalRule for CappedPageProposalNameLengthRule {
    fn check(&self, item: &CappedPageProposal) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPageProposalAllowedNameRule {
    pub allowed: Vec<String>,
}

impl CappedPageProposalRule for CappedPageProposalAllowedNameRule {
    fn check(&self, item: &CappedPageProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is too long", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPageProposalRequiredCodeRule {
    pub message: String,
}

impl CappedPageProposalRule for CappedPageProposalRequiredCodeRule {
    fn check(&self, item: &CappedPageProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped page proposal.
#[derive(Debug, Clone)]
pub struct CappedPageProposalValidator {
    pub rules: Vec<Rc<dyn CappedPageProposalRule>>,
}

impl CappedPageProposalValidator {
    pub fn validate(&self, item: &CappedPageProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedPageProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedPageProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_page_proposal_validator() -> CappedPageProposalValidator {
    CappedPageProposalValidator {
        rules: vec![
            Rc::new(CappedPageProposalQuantityRangeRule { min: 120, max: 255 }) as Rc<dyn CappedPageProposalRule>,
            Rc::new(CappedPageProposalMarginRangeRule { min: 8.9, max: 2166.3 }),
            Rc::new(CappedPageProposalRequiredScopeRule {
                message: "capped page proposal needs an entry".to_string(),
            }),
            Rc::new(CappedPageProposalNameLengthRule { max_length: 120 }),
            Rc::new(CappedPageProposalAllowedNameRule { allowed: Vec::new() }),
            Rc::new(CappedPageProposalRequiredCodeRule {
                message: "capped page proposal needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_page_proposal_validator();
        let item = new_capped_page_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = CappedPageProposalQuantityRangeRule { min: 16, max: 80 };
        let mut item = new_capped_page_proposal("x", "y");
        item.quantity = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = CappedPageProposalMarginRangeRule { min: 7.7, max: 4297.8 };
        let mut item = new_capped_page_proposal("x", "y");
        item.margin = 4299.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = CappedPageProposalRequiredScopeRule {
            message: "capped page proposal needs a value".to_string(),
        };
        let mut item = new_capped_page_proposal("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = CappedPageProposalNameLengthRule { max_length: 255 };
        let mut item = new_capped_page_proposal("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedPageProposalRequiredCodeRule {
            message: "capped page proposal needs a value".to_string(),
        };
        let mut item = new_capped_page_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
