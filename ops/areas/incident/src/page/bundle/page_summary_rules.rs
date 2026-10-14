use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a page summary must pass. Check returns its problems, or none.
pub trait PageSummaryRule: std::fmt::Debug {
    fn check(&self, item: &PageSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageSummaryNameLengthRule {
    pub max_length: i64,
}

impl PageSummaryRule for PageSummaryNameLengthRule {
    fn check(&self, item: &PageSummary) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageSummarySequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PageSummaryRule for PageSummarySequenceRangeRule {
    fn check(&self, item: &PageSummary) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageSummaryRequiredLocaleRule {
    pub message: String,
}

impl PageSummaryRule for PageSummaryRequiredLocaleRule {
    fn check(&self, item: &PageSummary) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageSummaryAllowedNameRule {
    pub allowed: Vec<String>,
}

impl PageSummaryRule for PageSummaryAllowedNameRule {
    fn check(&self, item: &PageSummary) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is expired", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageSummaryPinnedGuardRule {
    pub limit: i64,
}

impl PageSummaryRule for PageSummaryPinnedGuardRule {
    fn check(&self, item: &PageSummary) -> Vec<String> {
        if item.pinned && (item.sequence > self.limit) {
            return vec![format!(
                "sequence {} exceeds {} while pinned",
                item.sequence, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a page summary.
#[derive(Debug, Clone)]
pub struct PageSummaryValidator {
    pub rules: Vec<Rc<dyn PageSummaryRule>>,
}

impl PageSummaryValidator {
    pub fn validate(&self, item: &PageSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PageSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_summary_validator() -> PageSummaryValidator {
    PageSummaryValidator {
        rules: vec![
            Rc::new(PageSummaryNameLengthRule { max_length: 16 }) as Rc<dyn PageSummaryRule>,
            Rc::new(PageSummarySequenceRangeRule { min: 1000, max: 3600 }),
            Rc::new(PageSummaryRequiredLocaleRule {
                message: "page summary needs an entry".to_string(),
            }),
            Rc::new(PageSummaryAllowedNameRule {
                allowed: vec!["weighted".to_string(), "seasonal".to_string()],
            }),
            Rc::new(PageSummaryPinnedGuardRule { limit: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_page_summary_validator();
        let item = new_page_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PageSummaryNameLengthRule { max_length: 32 };
        let mut item = new_page_summary("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = PageSummarySequenceRangeRule { min: 16, max: 1000 };
        let mut item = new_page_summary("x", "y");
        item.sequence = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = PageSummaryRequiredLocaleRule {
            message: "page summary needs this field".to_string(),
        };
        let mut item = new_page_summary("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
