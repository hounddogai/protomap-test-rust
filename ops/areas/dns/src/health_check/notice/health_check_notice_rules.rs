use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a health check notice must pass. Check returns its problems, or none.
pub trait HealthCheckNoticeRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckNoticeAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl HealthCheckNoticeRule for HealthCheckNoticeAttemptsRangeRule {
    fn check(&self, item: &HealthCheckNotice) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckNoticeRequiredLabelRule {
    pub message: String,
}

impl HealthCheckNoticeRule for HealthCheckNoticeRequiredLabelRule {
    fn check(&self, item: &HealthCheckNotice) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckNoticeLabelLengthRule {
    pub max_length: i64,
}

impl HealthCheckNoticeRule for HealthCheckNoticeLabelLengthRule {
    fn check(&self, item: &HealthCheckNotice) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckNoticeCodePrefixRule {
    pub prefix: String,
}

impl HealthCheckNoticeRule for HealthCheckNoticeCodePrefixRule {
    fn check(&self, item: &HealthCheckNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckNoticeAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl HealthCheckNoticeRule for HealthCheckNoticeAllowedCodeRule {
    fn check(&self, item: &HealthCheckNotice) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check notice.
#[derive(Debug, Clone)]
pub struct HealthCheckNoticeValidator {
    pub rules: Vec<Rc<dyn HealthCheckNoticeRule>>,
}

impl HealthCheckNoticeValidator {
    pub fn validate(&self, item: &HealthCheckNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &HealthCheckNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_notice_validator() -> HealthCheckNoticeValidator {
    HealthCheckNoticeValidator {
        rules: vec![
            Rc::new(HealthCheckNoticeAttemptsRangeRule { min: 32, max: 64 }) as Rc<dyn HealthCheckNoticeRule>,
            Rc::new(HealthCheckNoticeRequiredLabelRule {
                message: "health check notice needs a value".to_string(),
            }),
            Rc::new(HealthCheckNoticeLabelLengthRule { max_length: 80 }),
            Rc::new(HealthCheckNoticeCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(HealthCheckNoticeAllowedCodeRule { allowed: Vec::new() }),
        ],
    }
}
