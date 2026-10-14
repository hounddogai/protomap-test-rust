use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy allowlist notice must pass. Check returns its problems, or none.
pub trait LegacyAllowlistNoticeRule: std::fmt::Debug {
    fn check(&self, item: &LegacyAllowlistNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistNoticeTitleLengthRule {
    pub max_length: i64,
}

impl LegacyAllowlistNoticeRule for LegacyAllowlistNoticeTitleLengthRule {
    fn check(&self, item: &LegacyAllowlistNotice) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistNoticeVisibleGuardRule {
    pub limit: f64,
}

impl LegacyAllowlistNoticeRule for LegacyAllowlistNoticeVisibleGuardRule {
    fn check(&self, item: &LegacyAllowlistNotice) -> Vec<String> {
        if item.visible && (item.weight > self.limit) {
            return vec![format!("weight {} exceeds {} while visible", item.weight, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistNoticeCodePrefixRule {
    pub prefix: String,
}

impl LegacyAllowlistNoticeRule for LegacyAllowlistNoticeCodePrefixRule {
    fn check(&self, item: &LegacyAllowlistNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy allowlist notice.
#[derive(Debug, Clone)]
pub struct LegacyAllowlistNoticeValidator {
    pub rules: Vec<Rc<dyn LegacyAllowlistNoticeRule>>,
}

impl LegacyAllowlistNoticeValidator {
    pub fn validate(&self, item: &LegacyAllowlistNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyAllowlistNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyAllowlistNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_allowlist_notice_validator() -> LegacyAllowlistNoticeValidator {
    LegacyAllowlistNoticeValidator {
        rules: vec![
            Rc::new(LegacyAllowlistNoticeTitleLengthRule { max_length: 16 }) as Rc<dyn LegacyAllowlistNoticeRule>,
            Rc::new(LegacyAllowlistNoticeVisibleGuardRule { limit: 4.4 }),
            Rc::new(LegacyAllowlistNoticeCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}
