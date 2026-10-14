use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional slo notice must pass. Check returns its problems, or none.
pub trait RegionalSloNoticeRule: std::fmt::Debug {
    fn check(&self, item: &RegionalSloNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSloNoticeAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RegionalSloNoticeRule for RegionalSloNoticeAllowedCodeRule {
    fn check(&self, item: &RegionalSloNotice) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSloNoticeRequiredCategoryRule {
    pub message: String,
}

impl RegionalSloNoticeRule for RegionalSloNoticeRequiredCategoryRule {
    fn check(&self, item: &RegionalSloNotice) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSloNoticeCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalSloNoticeRule for RegionalSloNoticeCapacityRangeRule {
    fn check(&self, item: &RegionalSloNotice) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSloNoticeCodeLengthRule {
    pub max_length: i64,
}

impl RegionalSloNoticeRule for RegionalSloNoticeCodeLengthRule {
    fn check(&self, item: &RegionalSloNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSloNoticeRequiredReviewerRule {
    pub message: String,
}

impl RegionalSloNoticeRule for RegionalSloNoticeRequiredReviewerRule {
    fn check(&self, item: &RegionalSloNotice) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSloNoticeUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalSloNoticeRule for RegionalSloNoticeUpdatedAtRangeRule {
    fn check(&self, item: &RegionalSloNotice) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional slo notice.
#[derive(Debug, Clone)]
pub struct RegionalSloNoticeValidator {
    pub rules: Vec<Rc<dyn RegionalSloNoticeRule>>,
}

impl RegionalSloNoticeValidator {
    pub fn validate(&self, item: &RegionalSloNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalSloNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalSloNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_slo_notice_validator() -> RegionalSloNoticeValidator {
    RegionalSloNoticeValidator {
        rules: vec![
            Rc::new(RegionalSloNoticeAllowedCodeRule { allowed: Vec::new() }) as Rc<dyn RegionalSloNoticeRule>,
            Rc::new(RegionalSloNoticeRequiredCategoryRule {
                message: "regional slo notice needs a value".to_string(),
            }),
            Rc::new(RegionalSloNoticeCapacityRangeRule { min: 120, max: 255 }),
            Rc::new(RegionalSloNoticeCodeLengthRule { max_length: 1000 }),
            Rc::new(RegionalSloNoticeRequiredReviewerRule {
                message: "regional slo notice needs a value".to_string(),
            }),
            Rc::new(RegionalSloNoticeUpdatedAtRangeRule { min: 120, max: 3600 }),
        ],
    }
}
