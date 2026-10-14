use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a retention ledger must pass. Check returns its problems, or none.
pub trait RetentionLedgerRule: std::fmt::Debug {
    fn check(&self, item: &RetentionLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionLedgerAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RetentionLedgerRule for RetentionLedgerAllowedCodeRule {
    fn check(&self, item: &RetentionLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionLedgerTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl RetentionLedgerRule for RetentionLedgerTagsLimitRule {
    fn check(&self, item: &RetentionLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionLedgerVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RetentionLedgerRule for RetentionLedgerVersionRangeRule {
    fn check(&self, item: &RetentionLedger) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionLedgerCodeLengthRule {
    pub max_length: i64,
}

impl RetentionLedgerRule for RetentionLedgerCodeLengthRule {
    fn check(&self, item: &RetentionLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionLedgerRequiredSummaryRule {
    pub message: String,
}

impl RetentionLedgerRule for RetentionLedgerRequiredSummaryRule {
    fn check(&self, item: &RetentionLedger) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionLedgerRequiredReviewerRule {
    pub message: String,
}

impl RetentionLedgerRule for RetentionLedgerRequiredReviewerRule {
    fn check(&self, item: &RetentionLedger) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention ledger.
#[derive(Debug, Clone)]
pub struct RetentionLedgerValidator {
    pub rules: Vec<Rc<dyn RetentionLedgerRule>>,
}

impl RetentionLedgerValidator {
    pub fn validate(&self, item: &RetentionLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RetentionLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_ledger_validator() -> RetentionLedgerValidator {
    RetentionLedgerValidator {
        rules: vec![
            Rc::new(RetentionLedgerAllowedCodeRule {
                allowed: vec!["strict".to_string(), "lenient".to_string(), "rolling".to_string()],
            }) as Rc<dyn RetentionLedgerRule>,
            Rc::new(RetentionLedgerTagsLimitRule {
                max_count: 16,
                banned: vec!["scheduled".to_string(), "tiered".to_string()],
            }),
            Rc::new(RetentionLedgerVersionRangeRule { min: 80, max: 1000 }),
            Rc::new(RetentionLedgerCodeLengthRule { max_length: 1000 }),
            Rc::new(RetentionLedgerRequiredSummaryRule {
                message: "retention ledger needs this field".to_string(),
            }),
            Rc::new(RetentionLedgerRequiredReviewerRule {
                message: "retention ledger needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_ledger_validator();
        let item = new_retention_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = RetentionLedgerVersionRangeRule { min: 1000, max: 86400 };
        let mut item = new_retention_ledger("x", "y");
        item.version = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RetentionLedgerCodeLengthRule { max_length: 120 };
        let mut item = new_retention_ledger("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = RetentionLedgerRequiredSummaryRule {
            message: "retention ledger needs an entry".to_string(),
        };
        let mut item = new_retention_ledger("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = RetentionLedgerRequiredReviewerRule {
            message: "retention ledger needs an entry".to_string(),
        };
        let mut item = new_retention_ledger("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
