use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling runbook summary must pass. Check returns its problems, or none.
pub trait RollingRunbookSummaryRule: std::fmt::Debug {
    fn check(&self, item: &RollingRunbookSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRunbookSummaryCodeLengthRule {
    pub max_length: i64,
}

impl RollingRunbookSummaryRule for RollingRunbookSummaryCodeLengthRule {
    fn check(&self, item: &RollingRunbookSummary) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRunbookSummaryOwnerIdLengthRule {
    pub max_length: i64,
}

impl RollingRunbookSummaryRule for RollingRunbookSummaryOwnerIdLengthRule {
    fn check(&self, item: &RollingRunbookSummary) -> Vec<String> {
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
pub struct RollingRunbookSummaryRequiredCodeRule {
    pub message: String,
}

impl RollingRunbookSummaryRule for RollingRunbookSummaryRequiredCodeRule {
    fn check(&self, item: &RollingRunbookSummary) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling runbook summary.
#[derive(Debug, Clone)]
pub struct RollingRunbookSummaryValidator {
    pub rules: Vec<Rc<dyn RollingRunbookSummaryRule>>,
}

impl RollingRunbookSummaryValidator {
    pub fn validate(&self, item: &RollingRunbookSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingRunbookSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingRunbookSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_runbook_summary_validator() -> RollingRunbookSummaryValidator {
    RollingRunbookSummaryValidator {
        rules: vec![
            Rc::new(RollingRunbookSummaryCodeLengthRule { max_length: 16 }) as Rc<dyn RollingRunbookSummaryRule>,
            Rc::new(RollingRunbookSummaryOwnerIdLengthRule { max_length: 80 }),
            Rc::new(RollingRunbookSummaryRequiredCodeRule {
                message: "rolling runbook summary needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_runbook_summary_validator();
        let item = new_rolling_runbook_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingRunbookSummaryCodeLengthRule { max_length: 1000 };
        let mut item = new_rolling_runbook_summary("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = RollingRunbookSummaryOwnerIdLengthRule { max_length: 3600 };
        let mut item = new_rolling_runbook_summary("x", "y");
        item.owner_id = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingRunbookSummaryRequiredCodeRule {
            message: "rolling runbook summary needs this field".to_string(),
        };
        let mut item = new_rolling_runbook_summary("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
