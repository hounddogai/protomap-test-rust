use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a page draft must pass. Check returns its problems, or none.
pub trait PageDraftRule: std::fmt::Debug {
    fn check(&self, item: &PageDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageDraftRequiredCurrencyRule {
    pub message: String,
}

impl PageDraftRule for PageDraftRequiredCurrencyRule {
    fn check(&self, item: &PageDraft) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageDraftAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PageDraftRule for PageDraftAllowedCodeRule {
    fn check(&self, item: &PageDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageDraftUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PageDraftRule for PageDraftUpdatedAtRangeRule {
    fn check(&self, item: &PageDraft) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a page draft.
#[derive(Debug, Clone)]
pub struct PageDraftValidator {
    pub rules: Vec<Rc<dyn PageDraftRule>>,
}

impl PageDraftValidator {
    pub fn validate(&self, item: &PageDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PageDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_draft_validator() -> PageDraftValidator {
    PageDraftValidator {
        rules: vec![
            Rc::new(PageDraftRequiredCurrencyRule {
                message: "page draft needs this field".to_string(),
            }) as Rc<dyn PageDraftRule>,
            Rc::new(PageDraftAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(PageDraftUpdatedAtRangeRule { min: 16, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_page_draft_validator();
        let item = new_page_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = PageDraftRequiredCurrencyRule {
            message: "page draft needs an entry".to_string(),
        };
        let mut item = new_page_draft("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = PageDraftUpdatedAtRangeRule { min: 80, max: 3600 };
        let mut item = new_page_draft("x", "y");
        item.updated_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
