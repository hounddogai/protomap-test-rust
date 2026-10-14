use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient measurement notice must pass. Check returns its problems, or none.
pub trait LenientMeasurementNoticeRule: std::fmt::Debug {
    fn check(&self, item: &LenientMeasurementNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMeasurementNoticeTitleLengthRule {
    pub max_length: i64,
}

impl LenientMeasurementNoticeRule for LenientMeasurementNoticeTitleLengthRule {
    fn check(&self, item: &LenientMeasurementNotice) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMeasurementNoticeOwnerIdLengthRule {
    pub max_length: i64,
}

impl LenientMeasurementNoticeRule for LenientMeasurementNoticeOwnerIdLengthRule {
    fn check(&self, item: &LenientMeasurementNotice) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMeasurementNoticeQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientMeasurementNoticeRule for LenientMeasurementNoticeQuantityRangeRule {
    fn check(&self, item: &LenientMeasurementNotice) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMeasurementNoticeRequiredOwnerIdRule {
    pub message: String,
}

impl LenientMeasurementNoticeRule for LenientMeasurementNoticeRequiredOwnerIdRule {
    fn check(&self, item: &LenientMeasurementNotice) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMeasurementNoticeRequiredTitleRule {
    pub message: String,
}

impl LenientMeasurementNoticeRule for LenientMeasurementNoticeRequiredTitleRule {
    fn check(&self, item: &LenientMeasurementNotice) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMeasurementNoticeCodePrefixRule {
    pub prefix: String,
}

impl LenientMeasurementNoticeRule for LenientMeasurementNoticeCodePrefixRule {
    fn check(&self, item: &LenientMeasurementNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient measurement notice.
#[derive(Debug, Clone)]
pub struct LenientMeasurementNoticeValidator {
    pub rules: Vec<Rc<dyn LenientMeasurementNoticeRule>>,
}

impl LenientMeasurementNoticeValidator {
    pub fn validate(&self, item: &LenientMeasurementNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientMeasurementNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientMeasurementNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_measurement_notice_validator() -> LenientMeasurementNoticeValidator {
    LenientMeasurementNoticeValidator {
        rules: vec![
            Rc::new(LenientMeasurementNoticeTitleLengthRule { max_length: 64 }) as Rc<dyn LenientMeasurementNoticeRule>,
            Rc::new(LenientMeasurementNoticeOwnerIdLengthRule { max_length: 120 }),
            Rc::new(LenientMeasurementNoticeQuantityRangeRule { min: 255, max: 1000 }),
            Rc::new(LenientMeasurementNoticeRequiredOwnerIdRule {
                message: "lenient measurement notice needs a value".to_string(),
            }),
            Rc::new(LenientMeasurementNoticeRequiredTitleRule {
                message: "lenient measurement notice needs this field".to_string(),
            }),
            Rc::new(LenientMeasurementNoticeCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}
