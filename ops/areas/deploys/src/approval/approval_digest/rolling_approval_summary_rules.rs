use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling approval summary must pass. Check returns its problems, or none.
pub trait RollingApprovalSummaryRule: std::fmt::Debug {
    fn check(&self, item: &RollingApprovalSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingApprovalSummaryRequiredNameRule {
    pub message: String,
}

impl RollingApprovalSummaryRule for RollingApprovalSummaryRequiredNameRule {
    fn check(&self, item: &RollingApprovalSummary) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingApprovalSummaryRequiredCodeRule {
    pub message: String,
}

impl RollingApprovalSummaryRule for RollingApprovalSummaryRequiredCodeRule {
    fn check(&self, item: &RollingApprovalSummary) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingApprovalSummaryChecksumLengthRule {
    pub max_length: i64,
}

impl RollingApprovalSummaryRule for RollingApprovalSummaryChecksumLengthRule {
    fn check(&self, item: &RollingApprovalSummary) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling approval summary.
#[derive(Debug, Clone)]
pub struct RollingApprovalSummaryValidator {
    pub rules: Vec<Rc<dyn RollingApprovalSummaryRule>>,
}

impl RollingApprovalSummaryValidator {
    pub fn validate(&self, item: &RollingApprovalSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingApprovalSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingApprovalSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_approval_summary_validator() -> RollingApprovalSummaryValidator {
    RollingApprovalSummaryValidator {
        rules: vec![
            Rc::new(RollingApprovalSummaryRequiredNameRule {
                message: "rolling approval summary needs a value".to_string(),
            }) as Rc<dyn RollingApprovalSummaryRule>,
            Rc::new(RollingApprovalSummaryRequiredCodeRule {
                message: "rolling approval summary needs an entry".to_string(),
            }),
            Rc::new(RollingApprovalSummaryChecksumLengthRule { max_length: 255 }),
        ],
    }
}
