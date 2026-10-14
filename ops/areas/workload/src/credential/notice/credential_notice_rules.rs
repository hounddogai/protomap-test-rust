use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a credential notice must pass. Check returns its problems, or none.
pub trait CredentialNoticeRule: std::fmt::Debug {
    fn check(&self, item: &CredentialNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialNoticeSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CredentialNoticeRule for CredentialNoticeSequenceRangeRule {
    fn check(&self, item: &CredentialNotice) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialNoticeCodePrefixRule {
    pub prefix: String,
}

impl CredentialNoticeRule for CredentialNoticeCodePrefixRule {
    fn check(&self, item: &CredentialNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialNoticeRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CredentialNoticeRule for CredentialNoticeRetriesRangeRule {
    fn check(&self, item: &CredentialNotice) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialNoticeAllowedContactEmailRule {
    pub allowed: Vec<String>,
}

impl CredentialNoticeRule for CredentialNoticeAllowedContactEmailRule {
    fn check(&self, item: &CredentialNotice) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.contact_email)))
        {
            return vec![format!("contact email {} is above maximum", item.contact_email)];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential notice.
#[derive(Debug, Clone)]
pub struct CredentialNoticeValidator {
    pub rules: Vec<Rc<dyn CredentialNoticeRule>>,
}

impl CredentialNoticeValidator {
    pub fn validate(&self, item: &CredentialNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CredentialNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_notice_validator() -> CredentialNoticeValidator {
    CredentialNoticeValidator {
        rules: vec![
            Rc::new(CredentialNoticeSequenceRangeRule { min: 32, max: 1000 }) as Rc<dyn CredentialNoticeRule>,
            Rc::new(CredentialNoticeCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(CredentialNoticeRetriesRangeRule { min: 255, max: 86400 }),
            Rc::new(CredentialNoticeAllowedContactEmailRule {
                allowed: vec!["global".to_string(), "lenient".to_string(), "regional".to_string()],
            }),
        ],
    }
}
