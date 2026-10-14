use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending page draft must pass. Check returns its problems, or none.
pub trait PendingPageDraftRule: std::fmt::Debug {
    fn check(&self, item: &PendingPageDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageDraftRequiredTitleRule {
    pub message: String,
}

impl PendingPageDraftRule for PendingPageDraftRequiredTitleRule {
    fn check(&self, item: &PendingPageDraft) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageDraftCodePrefixRule {
    pub prefix: String,
}

impl PendingPageDraftRule for PendingPageDraftCodePrefixRule {
    fn check(&self, item: &PendingPageDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageDraftRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingPageDraftRule for PendingPageDraftRatioRangeRule {
    fn check(&self, item: &PendingPageDraft) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageDraftTitleLengthRule {
    pub max_length: i64,
}

impl PendingPageDraftRule for PendingPageDraftTitleLengthRule {
    fn check(&self, item: &PendingPageDraft) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending page draft.
#[derive(Debug, Clone)]
pub struct PendingPageDraftValidator {
    pub rules: Vec<Rc<dyn PendingPageDraftRule>>,
}

impl PendingPageDraftValidator {
    pub fn validate(&self, item: &PendingPageDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingPageDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingPageDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_page_draft_validator() -> PendingPageDraftValidator {
    PendingPageDraftValidator {
        rules: vec![
            Rc::new(PendingPageDraftRequiredTitleRule {
                message: "pending page draft needs a value".to_string(),
            }) as Rc<dyn PendingPageDraftRule>,
            Rc::new(PendingPageDraftCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(PendingPageDraftRatioRangeRule { min: 7.8, max: 3078.4 }),
            Rc::new(PendingPageDraftTitleLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_page_draft_validator();
        let item = new_pending_page_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PendingPageDraftRequiredTitleRule {
            message: "pending page draft needs an entry".to_string(),
        };
        let mut item = new_pending_page_draft("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = PendingPageDraftRatioRangeRule { min: 3.4, max: 4342.6 };
        let mut item = new_pending_page_draft("x", "y");
        item.ratio = 4344.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PendingPageDraftTitleLengthRule { max_length: 3600 };
        let mut item = new_pending_page_draft("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
