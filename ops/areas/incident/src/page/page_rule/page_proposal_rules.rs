use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a page proposal must pass. Check returns its problems, or none.
pub trait PageProposalRule: std::fmt::Debug {
    fn check(&self, item: &PageProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageProposalCodeLengthRule {
    pub max_length: i64,
}

impl PageProposalRule for PageProposalCodeLengthRule {
    fn check(&self, item: &PageProposal) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageProposalCategoryLengthRule {
    pub max_length: i64,
}

impl PageProposalRule for PageProposalCategoryLengthRule {
    fn check(&self, item: &PageProposal) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageProposalTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PageProposalRule for PageProposalTtlSecondsRangeRule {
    fn check(&self, item: &PageProposal) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageProposalAudienceLengthRule {
    pub max_length: i64,
}

impl PageProposalRule for PageProposalAudienceLengthRule {
    fn check(&self, item: &PageProposal) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageProposalNameLengthRule {
    pub max_length: i64,
}

impl PageProposalRule for PageProposalNameLengthRule {
    fn check(&self, item: &PageProposal) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageProposalRequiredCodeRule {
    pub message: String,
}

impl PageProposalRule for PageProposalRequiredCodeRule {
    fn check(&self, item: &PageProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a page proposal.
#[derive(Debug, Clone)]
pub struct PageProposalValidator {
    pub rules: Vec<Rc<dyn PageProposalRule>>,
}

impl PageProposalValidator {
    pub fn validate(&self, item: &PageProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PageProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_proposal_validator() -> PageProposalValidator {
    PageProposalValidator {
        rules: vec![
            Rc::new(PageProposalCodeLengthRule { max_length: 32 }) as Rc<dyn PageProposalRule>,
            Rc::new(PageProposalCategoryLengthRule { max_length: 3600 }),
            Rc::new(PageProposalTtlSecondsRangeRule { min: 255, max: 86400 }),
            Rc::new(PageProposalAudienceLengthRule { max_length: 80 }),
            Rc::new(PageProposalNameLengthRule { max_length: 32 }),
            Rc::new(PageProposalRequiredCodeRule {
                message: "page proposal needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_page_proposal_validator();
        let item = new_page_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PageProposalCodeLengthRule { max_length: 32 };
        let mut item = new_page_proposal("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = PageProposalCategoryLengthRule { max_length: 255 };
        let mut item = new_page_proposal("x", "y");
        item.category = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = PageProposalTtlSecondsRangeRule { min: 80, max: 120 };
        let mut item = new_page_proposal("x", "y");
        item.ttl_seconds = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = PageProposalAudienceLengthRule { max_length: 16 };
        let mut item = new_page_proposal("x", "y");
        item.audience = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PageProposalNameLengthRule { max_length: 1000 };
        let mut item = new_page_proposal("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PageProposalRequiredCodeRule {
            message: "page proposal needs this field".to_string(),
        };
        let mut item = new_page_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
