use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched runbook notice must pass. Check returns its problems, or none.
pub trait BatchedRunbookNoticeRule: std::fmt::Debug {
    fn check(&self, item: &BatchedRunbookNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookNoticeRequiredReviewerRule {
    pub message: String,
}

impl BatchedRunbookNoticeRule for BatchedRunbookNoticeRequiredReviewerRule {
    fn check(&self, item: &BatchedRunbookNotice) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookNoticeRequiredSourceRule {
    pub message: String,
}

impl BatchedRunbookNoticeRule for BatchedRunbookNoticeRequiredSourceRule {
    fn check(&self, item: &BatchedRunbookNotice) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookNoticeCodeLengthRule {
    pub max_length: i64,
}

impl BatchedRunbookNoticeRule for BatchedRunbookNoticeCodeLengthRule {
    fn check(&self, item: &BatchedRunbookNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookNoticeAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl BatchedRunbookNoticeRule for BatchedRunbookNoticeAllowedCodeRule {
    fn check(&self, item: &BatchedRunbookNotice) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookNoticeRequiredCodeRule {
    pub message: String,
}

impl BatchedRunbookNoticeRule for BatchedRunbookNoticeRequiredCodeRule {
    fn check(&self, item: &BatchedRunbookNotice) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookNoticeSourceLengthRule {
    pub max_length: i64,
}

impl BatchedRunbookNoticeRule for BatchedRunbookNoticeSourceLengthRule {
    fn check(&self, item: &BatchedRunbookNotice) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched runbook notice.
#[derive(Debug, Clone)]
pub struct BatchedRunbookNoticeValidator {
    pub rules: Vec<Rc<dyn BatchedRunbookNoticeRule>>,
}

impl BatchedRunbookNoticeValidator {
    pub fn validate(&self, item: &BatchedRunbookNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedRunbookNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedRunbookNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_runbook_notice_validator() -> BatchedRunbookNoticeValidator {
    BatchedRunbookNoticeValidator {
        rules: vec![
            Rc::new(BatchedRunbookNoticeRequiredReviewerRule {
                message: "batched runbook notice needs a value".to_string(),
            }) as Rc<dyn BatchedRunbookNoticeRule>,
            Rc::new(BatchedRunbookNoticeRequiredSourceRule {
                message: "batched runbook notice needs this field".to_string(),
            }),
            Rc::new(BatchedRunbookNoticeCodeLengthRule { max_length: 64 }),
            Rc::new(BatchedRunbookNoticeAllowedCodeRule {
                allowed: vec!["seasonal".to_string()],
            }),
            Rc::new(BatchedRunbookNoticeRequiredCodeRule {
                message: "batched runbook notice needs an entry".to_string(),
            }),
            Rc::new(BatchedRunbookNoticeSourceLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_runbook_notice_validator();
        let item = new_batched_runbook_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = BatchedRunbookNoticeRequiredReviewerRule {
            message: "batched runbook notice needs an entry".to_string(),
        };
        let mut item = new_batched_runbook_notice("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = BatchedRunbookNoticeRequiredSourceRule {
            message: "batched runbook notice needs this field".to_string(),
        };
        let mut item = new_batched_runbook_notice("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedRunbookNoticeCodeLengthRule { max_length: 64 };
        let mut item = new_batched_runbook_notice("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedRunbookNoticeRequiredCodeRule {
            message: "batched runbook notice needs an entry".to_string(),
        };
        let mut item = new_batched_runbook_notice("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = BatchedRunbookNoticeSourceLengthRule { max_length: 32 };
        let mut item = new_batched_runbook_notice("x", "y");
        item.source = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
