use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped retention notice must pass. Check returns its problems, or none.
pub trait CappedRetentionNoticeRule: std::fmt::Debug {
    fn check(&self, item: &CappedRetentionNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRetentionNoticeCodePrefixRule {
    pub prefix: String,
}

impl CappedRetentionNoticeRule for CappedRetentionNoticeCodePrefixRule {
    fn check(&self, item: &CappedRetentionNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRetentionNoticeLabelLengthRule {
    pub max_length: i64,
}

impl CappedRetentionNoticeRule for CappedRetentionNoticeLabelLengthRule {
    fn check(&self, item: &CappedRetentionNotice) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRetentionNoticeUnitLengthRule {
    pub max_length: i64,
}

impl CappedRetentionNoticeRule for CappedRetentionNoticeUnitLengthRule {
    fn check(&self, item: &CappedRetentionNotice) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped retention notice.
#[derive(Debug, Clone)]
pub struct CappedRetentionNoticeValidator {
    pub rules: Vec<Rc<dyn CappedRetentionNoticeRule>>,
}

impl CappedRetentionNoticeValidator {
    pub fn validate(&self, item: &CappedRetentionNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedRetentionNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedRetentionNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_retention_notice_validator() -> CappedRetentionNoticeValidator {
    CappedRetentionNoticeValidator {
        rules: vec![
            Rc::new(CappedRetentionNoticeCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn CappedRetentionNoticeRule>,
            Rc::new(CappedRetentionNoticeLabelLengthRule { max_length: 1000 }),
            Rc::new(CappedRetentionNoticeUnitLengthRule { max_length: 255 }),
        ],
    }
}
