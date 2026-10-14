use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped approval notice must pass. Check returns its problems, or none.
pub trait CappedApprovalNoticeRule: std::fmt::Debug {
    fn check(&self, item: &CappedApprovalNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedApprovalNoticeRequiredCodeRule {
    pub message: String,
}

impl CappedApprovalNoticeRule for CappedApprovalNoticeRequiredCodeRule {
    fn check(&self, item: &CappedApprovalNotice) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedApprovalNoticeTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedApprovalNoticeRule for CappedApprovalNoticeTtlSecondsRangeRule {
    fn check(&self, item: &CappedApprovalNotice) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedApprovalNoticeLocaleLengthRule {
    pub max_length: i64,
}

impl CappedApprovalNoticeRule for CappedApprovalNoticeLocaleLengthRule {
    fn check(&self, item: &CappedApprovalNotice) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedApprovalNoticeRequiredUnitRule {
    pub message: String,
}

impl CappedApprovalNoticeRule for CappedApprovalNoticeRequiredUnitRule {
    fn check(&self, item: &CappedApprovalNotice) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedApprovalNoticeExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedApprovalNoticeRule for CappedApprovalNoticeExpiresAtRangeRule {
    fn check(&self, item: &CappedApprovalNotice) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a capped approval notice.
#[derive(Debug, Clone)]
pub struct CappedApprovalNoticeValidator {
    pub rules: Vec<Rc<dyn CappedApprovalNoticeRule>>,
}

impl CappedApprovalNoticeValidator {
    pub fn validate(&self, item: &CappedApprovalNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedApprovalNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedApprovalNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_approval_notice_validator() -> CappedApprovalNoticeValidator {
    CappedApprovalNoticeValidator {
        rules: vec![
            Rc::new(CappedApprovalNoticeRequiredCodeRule {
                message: "capped approval notice needs a value".to_string(),
            }) as Rc<dyn CappedApprovalNoticeRule>,
            Rc::new(CappedApprovalNoticeTtlSecondsRangeRule { min: 64, max: 1000 }),
            Rc::new(CappedApprovalNoticeLocaleLengthRule { max_length: 3600 }),
            Rc::new(CappedApprovalNoticeRequiredUnitRule {
                message: "capped approval notice needs this field".to_string(),
            }),
            Rc::new(CappedApprovalNoticeExpiresAtRangeRule { min: 120, max: 86400 }),
        ],
    }
}
