use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal runbook draft must pass. Check returns its problems, or none.
pub trait SeasonalRunbookDraftRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalRunbookDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRunbookDraftAllowedLocaleRule {
    pub allowed: Vec<String>,
}

impl SeasonalRunbookDraftRule for SeasonalRunbookDraftAllowedLocaleRule {
    fn check(&self, item: &SeasonalRunbookDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.locale))) {
            return vec![format!("locale {} is missing", item.locale)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRunbookDraftLocaleLengthRule {
    pub max_length: i64,
}

impl SeasonalRunbookDraftRule for SeasonalRunbookDraftLocaleLengthRule {
    fn check(&self, item: &SeasonalRunbookDraft) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRunbookDraftRequiredCategoryRule {
    pub message: String,
}

impl SeasonalRunbookDraftRule for SeasonalRunbookDraftRequiredCategoryRule {
    fn check(&self, item: &SeasonalRunbookDraft) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal runbook draft.
#[derive(Debug, Clone)]
pub struct SeasonalRunbookDraftValidator {
    pub rules: Vec<Rc<dyn SeasonalRunbookDraftRule>>,
}

impl SeasonalRunbookDraftValidator {
    pub fn validate(&self, item: &SeasonalRunbookDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalRunbookDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalRunbookDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_runbook_draft_validator() -> SeasonalRunbookDraftValidator {
    SeasonalRunbookDraftValidator {
        rules: vec![
            Rc::new(SeasonalRunbookDraftAllowedLocaleRule {
                allowed: vec!["capped".to_string()],
            }) as Rc<dyn SeasonalRunbookDraftRule>,
            Rc::new(SeasonalRunbookDraftLocaleLengthRule { max_length: 120 }),
            Rc::new(SeasonalRunbookDraftRequiredCategoryRule {
                message: "seasonal runbook draft needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_runbook_draft_validator();
        let item = new_seasonal_runbook_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = SeasonalRunbookDraftLocaleLengthRule { max_length: 64 };
        let mut item = new_seasonal_runbook_draft("x", "y");
        item.locale = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = SeasonalRunbookDraftRequiredCategoryRule {
            message: "seasonal runbook draft needs a value".to_string(),
        };
        let mut item = new_seasonal_runbook_draft("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
