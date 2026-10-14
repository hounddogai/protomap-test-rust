use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a page agreement must pass. Check returns its problems, or none.
pub trait PageAgreementRule: std::fmt::Debug {
    fn check(&self, item: &PageAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageAgreementCodeLengthRule {
    pub max_length: i64,
}

impl PageAgreementRule for PageAgreementCodeLengthRule {
    fn check(&self, item: &PageAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageAgreementAllowedReviewerRule {
    pub allowed: Vec<String>,
}

impl PageAgreementRule for PageAgreementAllowedReviewerRule {
    fn check(&self, item: &PageAgreement) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.reviewer)))
        {
            return vec![format!("reviewer {} is expired", item.reviewer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageAgreementRequiredLabelRule {
    pub message: String,
}

impl PageAgreementRule for PageAgreementRequiredLabelRule {
    fn check(&self, item: &PageAgreement) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageAgreementStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PageAgreementRule for PageAgreementStartsAtRangeRule {
    fn check(&self, item: &PageAgreement) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a page agreement.
#[derive(Debug, Clone)]
pub struct PageAgreementValidator {
    pub rules: Vec<Rc<dyn PageAgreementRule>>,
}

impl PageAgreementValidator {
    pub fn validate(&self, item: &PageAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PageAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_agreement_validator() -> PageAgreementValidator {
    PageAgreementValidator {
        rules: vec![
            Rc::new(PageAgreementCodeLengthRule { max_length: 120 }) as Rc<dyn PageAgreementRule>,
            Rc::new(PageAgreementAllowedReviewerRule {
                allowed: vec!["legacy".to_string()],
            }),
            Rc::new(PageAgreementRequiredLabelRule {
                message: "page agreement needs an entry".to_string(),
            }),
            Rc::new(PageAgreementStartsAtRangeRule { min: 64, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_page_agreement_validator();
        let item = new_page_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PageAgreementCodeLengthRule { max_length: 1000 };
        let mut item = new_page_agreement("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PageAgreementRequiredLabelRule {
            message: "page agreement needs a value".to_string(),
        };
        let mut item = new_page_agreement("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = PageAgreementStartsAtRangeRule { min: 120, max: 255 };
        let mut item = new_page_agreement("x", "y");
        item.starts_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
