use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional tenant notice must pass. Check returns its problems, or none.
pub trait RegionalTenantNoticeRule: std::fmt::Debug {
    fn check(&self, item: &RegionalTenantNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTenantNoticeNamespaceLengthRule {
    pub max_length: i64,
}

impl RegionalTenantNoticeRule for RegionalTenantNoticeNamespaceLengthRule {
    fn check(&self, item: &RegionalTenantNotice) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTenantNoticeBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalTenantNoticeRule for RegionalTenantNoticeBatchSizeRangeRule {
    fn check(&self, item: &RegionalTenantNotice) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTenantNoticeCodeLengthRule {
    pub max_length: i64,
}

impl RegionalTenantNoticeRule for RegionalTenantNoticeCodeLengthRule {
    fn check(&self, item: &RegionalTenantNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTenantNoticeRequiredCategoryRule {
    pub message: String,
}

impl RegionalTenantNoticeRule for RegionalTenantNoticeRequiredCategoryRule {
    fn check(&self, item: &RegionalTenantNotice) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional tenant notice.
#[derive(Debug, Clone)]
pub struct RegionalTenantNoticeValidator {
    pub rules: Vec<Rc<dyn RegionalTenantNoticeRule>>,
}

impl RegionalTenantNoticeValidator {
    pub fn validate(&self, item: &RegionalTenantNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalTenantNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalTenantNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_tenant_notice_validator() -> RegionalTenantNoticeValidator {
    RegionalTenantNoticeValidator {
        rules: vec![
            Rc::new(RegionalTenantNoticeNamespaceLengthRule { max_length: 1000 }) as Rc<dyn RegionalTenantNoticeRule>,
            Rc::new(RegionalTenantNoticeBatchSizeRangeRule { min: 120, max: 255 }),
            Rc::new(RegionalTenantNoticeCodeLengthRule { max_length: 255 }),
            Rc::new(RegionalTenantNoticeRequiredCategoryRule {
                message: "regional tenant notice needs an entry".to_string(),
            }),
        ],
    }
}
