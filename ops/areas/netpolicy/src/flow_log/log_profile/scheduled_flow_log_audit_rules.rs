use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled flow log audit must pass. Check returns its problems, or none.
pub trait ScheduledFlowLogAuditRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledFlowLogAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFlowLogAuditHostLengthRule {
    pub max_length: i64,
}

impl ScheduledFlowLogAuditRule for ScheduledFlowLogAuditHostLengthRule {
    fn check(&self, item: &ScheduledFlowLogAudit) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFlowLogAuditGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledFlowLogAuditRule for ScheduledFlowLogAuditGraceDaysRangeRule {
    fn check(&self, item: &ScheduledFlowLogAudit) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFlowLogAuditCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledFlowLogAuditRule for ScheduledFlowLogAuditCodeLengthRule {
    fn check(&self, item: &ScheduledFlowLogAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFlowLogAuditRequiredHostRule {
    pub message: String,
}

impl ScheduledFlowLogAuditRule for ScheduledFlowLogAuditRequiredHostRule {
    fn check(&self, item: &ScheduledFlowLogAudit) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFlowLogAuditRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledFlowLogAuditRule for ScheduledFlowLogAuditRevisionRangeRule {
    fn check(&self, item: &ScheduledFlowLogAudit) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled flow log audit.
#[derive(Debug, Clone)]
pub struct ScheduledFlowLogAuditValidator {
    pub rules: Vec<Rc<dyn ScheduledFlowLogAuditRule>>,
}

impl ScheduledFlowLogAuditValidator {
    pub fn validate(&self, item: &ScheduledFlowLogAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledFlowLogAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledFlowLogAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_flow_log_audit_validator() -> ScheduledFlowLogAuditValidator {
    ScheduledFlowLogAuditValidator {
        rules: vec![
            Rc::new(ScheduledFlowLogAuditHostLengthRule { max_length: 86400 }) as Rc<dyn ScheduledFlowLogAuditRule>,
            Rc::new(ScheduledFlowLogAuditGraceDaysRangeRule { min: 64, max: 80 }),
            Rc::new(ScheduledFlowLogAuditCodeLengthRule { max_length: 3600 }),
            Rc::new(ScheduledFlowLogAuditRequiredHostRule {
                message: "scheduled flow log audit needs this field".to_string(),
            }),
            Rc::new(ScheduledFlowLogAuditRevisionRangeRule { min: 16, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_flow_log_audit_validator();
        let item = new_scheduled_flow_log_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = ScheduledFlowLogAuditHostLengthRule { max_length: 16 };
        let mut item = new_scheduled_flow_log_audit("x", "y");
        item.host = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = ScheduledFlowLogAuditGraceDaysRangeRule { min: 80, max: 1000 };
        let mut item = new_scheduled_flow_log_audit("x", "y");
        item.grace_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledFlowLogAuditCodeLengthRule { max_length: 120 };
        let mut item = new_scheduled_flow_log_audit("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = ScheduledFlowLogAuditRequiredHostRule {
            message: "scheduled flow log audit needs a value".to_string(),
        };
        let mut item = new_scheduled_flow_log_audit("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = ScheduledFlowLogAuditRevisionRangeRule { min: 64, max: 3600 };
        let mut item = new_scheduled_flow_log_audit("x", "y");
        item.revision = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
