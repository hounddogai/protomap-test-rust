use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal credential notice must pass. Check returns its problems, or none.
pub trait SeasonalCredentialNoticeRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalCredentialNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialNoticeCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalCredentialNoticeRule for SeasonalCredentialNoticeCodeLengthRule {
    fn check(&self, item: &SeasonalCredentialNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialNoticeLabelLengthRule {
    pub max_length: i64,
}

impl SeasonalCredentialNoticeRule for SeasonalCredentialNoticeLabelLengthRule {
    fn check(&self, item: &SeasonalCredentialNotice) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialNoticeCodePrefixRule {
    pub prefix: String,
}

impl SeasonalCredentialNoticeRule for SeasonalCredentialNoticeCodePrefixRule {
    fn check(&self, item: &SeasonalCredentialNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal credential notice.
#[derive(Debug, Clone)]
pub struct SeasonalCredentialNoticeValidator {
    pub rules: Vec<Rc<dyn SeasonalCredentialNoticeRule>>,
}

impl SeasonalCredentialNoticeValidator {
    pub fn validate(&self, item: &SeasonalCredentialNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalCredentialNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalCredentialNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_credential_notice_validator() -> SeasonalCredentialNoticeValidator {
    SeasonalCredentialNoticeValidator {
        rules: vec![
            Rc::new(SeasonalCredentialNoticeCodeLengthRule { max_length: 32 }) as Rc<dyn SeasonalCredentialNoticeRule>,
            Rc::new(SeasonalCredentialNoticeLabelLengthRule { max_length: 80 }),
            Rc::new(SeasonalCredentialNoticeCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}
