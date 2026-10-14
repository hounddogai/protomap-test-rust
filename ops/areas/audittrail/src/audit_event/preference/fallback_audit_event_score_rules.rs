use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback audit event score must pass. Check returns its problems, or none.
pub trait FallbackAuditEventScoreRule: std::fmt::Debug {
    fn check(&self, item: &FallbackAuditEventScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAuditEventScoreRequiredTitleRule {
    pub message: String,
}

impl FallbackAuditEventScoreRule for FallbackAuditEventScoreRequiredTitleRule {
    fn check(&self, item: &FallbackAuditEventScore) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAuditEventScoreTitleLengthRule {
    pub max_length: i64,
}

impl FallbackAuditEventScoreRule for FallbackAuditEventScoreTitleLengthRule {
    fn check(&self, item: &FallbackAuditEventScore) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAuditEventScoreBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackAuditEventScoreRule for FallbackAuditEventScoreBurstRangeRule {
    fn check(&self, item: &FallbackAuditEventScore) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAuditEventScoreCodeLengthRule {
    pub max_length: i64,
}

impl FallbackAuditEventScoreRule for FallbackAuditEventScoreCodeLengthRule {
    fn check(&self, item: &FallbackAuditEventScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAuditEventScoreRequiredIssuerRule {
    pub message: String,
}

impl FallbackAuditEventScoreRule for FallbackAuditEventScoreRequiredIssuerRule {
    fn check(&self, item: &FallbackAuditEventScore) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback audit event score.
#[derive(Debug, Clone)]
pub struct FallbackAuditEventScoreValidator {
    pub rules: Vec<Rc<dyn FallbackAuditEventScoreRule>>,
}

impl FallbackAuditEventScoreValidator {
    pub fn validate(&self, item: &FallbackAuditEventScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackAuditEventScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackAuditEventScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_audit_event_score_validator() -> FallbackAuditEventScoreValidator {
    FallbackAuditEventScoreValidator {
        rules: vec![
            Rc::new(FallbackAuditEventScoreRequiredTitleRule {
                message: "fallback audit event score needs this field".to_string(),
            }) as Rc<dyn FallbackAuditEventScoreRule>,
            Rc::new(FallbackAuditEventScoreTitleLengthRule { max_length: 120 }),
            Rc::new(FallbackAuditEventScoreBurstRangeRule { min: 255, max: 86400 }),
            Rc::new(FallbackAuditEventScoreCodeLengthRule { max_length: 3600 }),
            Rc::new(FallbackAuditEventScoreRequiredIssuerRule {
                message: "fallback audit event score needs a value".to_string(),
            }),
        ],
    }
}
