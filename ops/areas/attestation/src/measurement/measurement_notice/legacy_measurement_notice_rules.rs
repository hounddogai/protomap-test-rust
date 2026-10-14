use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy measurement notice must pass. Check returns its problems, or none.
pub trait LegacyMeasurementNoticeRule: std::fmt::Debug {
    fn check(&self, item: &LegacyMeasurementNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementNoticeRequiredCodeRule {
    pub message: String,
}

impl LegacyMeasurementNoticeRule for LegacyMeasurementNoticeRequiredCodeRule {
    fn check(&self, item: &LegacyMeasurementNotice) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementNoticeRequiredTitleRule {
    pub message: String,
}

impl LegacyMeasurementNoticeRule for LegacyMeasurementNoticeRequiredTitleRule {
    fn check(&self, item: &LegacyMeasurementNotice) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementNoticeFingerprintLengthRule {
    pub max_length: i64,
}

impl LegacyMeasurementNoticeRule for LegacyMeasurementNoticeFingerprintLengthRule {
    fn check(&self, item: &LegacyMeasurementNotice) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementNoticeCodePrefixRule {
    pub prefix: String,
}

impl LegacyMeasurementNoticeRule for LegacyMeasurementNoticeCodePrefixRule {
    fn check(&self, item: &LegacyMeasurementNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementNoticeTitleLengthRule {
    pub max_length: i64,
}

impl LegacyMeasurementNoticeRule for LegacyMeasurementNoticeTitleLengthRule {
    fn check(&self, item: &LegacyMeasurementNotice) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementNoticeQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyMeasurementNoticeRule for LegacyMeasurementNoticeQuotaUnitsRangeRule {
    fn check(&self, item: &LegacyMeasurementNotice) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy measurement notice.
#[derive(Debug, Clone)]
pub struct LegacyMeasurementNoticeValidator {
    pub rules: Vec<Rc<dyn LegacyMeasurementNoticeRule>>,
}

impl LegacyMeasurementNoticeValidator {
    pub fn validate(&self, item: &LegacyMeasurementNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyMeasurementNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyMeasurementNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_measurement_notice_validator() -> LegacyMeasurementNoticeValidator {
    LegacyMeasurementNoticeValidator {
        rules: vec![
            Rc::new(LegacyMeasurementNoticeRequiredCodeRule {
                message: "legacy measurement notice needs an entry".to_string(),
            }) as Rc<dyn LegacyMeasurementNoticeRule>,
            Rc::new(LegacyMeasurementNoticeRequiredTitleRule {
                message: "legacy measurement notice needs an entry".to_string(),
            }),
            Rc::new(LegacyMeasurementNoticeFingerprintLengthRule { max_length: 80 }),
            Rc::new(LegacyMeasurementNoticeCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(LegacyMeasurementNoticeTitleLengthRule { max_length: 32 }),
            Rc::new(LegacyMeasurementNoticeQuotaUnitsRangeRule { min: 80, max: 3600 }),
        ],
    }
}
