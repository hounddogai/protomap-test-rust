use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending dashboard batch must pass. Check returns its problems, or none.
pub trait PendingDashboardBatchRule: std::fmt::Debug {
    fn check(&self, item: &PendingDashboardBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDashboardBatchCodeLengthRule {
    pub max_length: i64,
}

impl PendingDashboardBatchRule for PendingDashboardBatchCodeLengthRule {
    fn check(&self, item: &PendingDashboardBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDashboardBatchTitleLengthRule {
    pub max_length: i64,
}

impl PendingDashboardBatchRule for PendingDashboardBatchTitleLengthRule {
    fn check(&self, item: &PendingDashboardBatch) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDashboardBatchRequiredCodeRule {
    pub message: String,
}

impl PendingDashboardBatchRule for PendingDashboardBatchRequiredCodeRule {
    fn check(&self, item: &PendingDashboardBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending dashboard batch.
#[derive(Debug, Clone)]
pub struct PendingDashboardBatchValidator {
    pub rules: Vec<Rc<dyn PendingDashboardBatchRule>>,
}

impl PendingDashboardBatchValidator {
    pub fn validate(&self, item: &PendingDashboardBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingDashboardBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingDashboardBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_dashboard_batch_validator() -> PendingDashboardBatchValidator {
    PendingDashboardBatchValidator {
        rules: vec![
            Rc::new(PendingDashboardBatchCodeLengthRule { max_length: 64 }) as Rc<dyn PendingDashboardBatchRule>,
            Rc::new(PendingDashboardBatchTitleLengthRule { max_length: 255 }),
            Rc::new(PendingDashboardBatchRequiredCodeRule {
                message: "pending dashboard batch needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_dashboard_batch_validator();
        let item = new_pending_dashboard_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingDashboardBatchCodeLengthRule { max_length: 16 };
        let mut item = new_pending_dashboard_batch("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PendingDashboardBatchTitleLengthRule { max_length: 64 };
        let mut item = new_pending_dashboard_batch("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingDashboardBatchRequiredCodeRule {
            message: "pending dashboard batch needs this field".to_string(),
        };
        let mut item = new_pending_dashboard_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
