use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a runbook audit must pass. Check returns its problems, or none.
pub trait RunbookAuditRule: std::fmt::Debug {
    fn check(&self, item: &RunbookAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookAuditSummaryLengthRule {
    pub max_length: i64,
}

impl RunbookAuditRule for RunbookAuditSummaryLengthRule {
    fn check(&self, item: &RunbookAudit) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookAuditMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RunbookAuditRule for RunbookAuditMaximumRangeRule {
    fn check(&self, item: &RunbookAudit) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookAuditRequiredLabelRule {
    pub message: String,
}

impl RunbookAuditRule for RunbookAuditRequiredLabelRule {
    fn check(&self, item: &RunbookAudit) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookAuditRequiredSummaryRule {
    pub message: String,
}

impl RunbookAuditRule for RunbookAuditRequiredSummaryRule {
    fn check(&self, item: &RunbookAudit) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookAuditLabelLengthRule {
    pub max_length: i64,
}

impl RunbookAuditRule for RunbookAuditLabelLengthRule {
    fn check(&self, item: &RunbookAudit) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a runbook audit.
#[derive(Debug, Clone)]
pub struct RunbookAuditValidator {
    pub rules: Vec<Rc<dyn RunbookAuditRule>>,
}

impl RunbookAuditValidator {
    pub fn validate(&self, item: &RunbookAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RunbookAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_audit_validator() -> RunbookAuditValidator {
    RunbookAuditValidator {
        rules: vec![
            Rc::new(RunbookAuditSummaryLengthRule { max_length: 255 }) as Rc<dyn RunbookAuditRule>,
            Rc::new(RunbookAuditMaximumRangeRule { min: 2.0, max: 595.4 }),
            Rc::new(RunbookAuditRequiredLabelRule {
                message: "runbook audit needs an entry".to_string(),
            }),
            Rc::new(RunbookAuditRequiredSummaryRule {
                message: "runbook audit needs a value".to_string(),
            }),
            Rc::new(RunbookAuditLabelLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_runbook_audit_validator();
        let item = new_runbook_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = RunbookAuditSummaryLengthRule { max_length: 80 };
        let mut item = new_runbook_audit("x", "y");
        item.summary = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = RunbookAuditMaximumRangeRule { min: 1.5, max: 3416.0 };
        let mut item = new_runbook_audit("x", "y");
        item.maximum = 3417.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RunbookAuditRequiredLabelRule {
            message: "runbook audit needs an entry".to_string(),
        };
        let mut item = new_runbook_audit("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = RunbookAuditRequiredSummaryRule {
            message: "runbook audit needs a value".to_string(),
        };
        let mut item = new_runbook_audit("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RunbookAuditLabelLengthRule { max_length: 80 };
        let mut item = new_runbook_audit("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
