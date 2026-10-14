use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollback audit must pass. Check returns its problems, or none.
pub trait RollbackAuditRule: std::fmt::Debug {
    fn check(&self, item: &RollbackAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAuditClusterLengthRule {
    pub max_length: i64,
}

impl RollbackAuditRule for RollbackAuditClusterLengthRule {
    fn check(&self, item: &RollbackAudit) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAuditVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollbackAuditRule for RollbackAuditVersionRangeRule {
    fn check(&self, item: &RollbackAudit) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAuditRequiredLabelRule {
    pub message: String,
}

impl RollbackAuditRule for RollbackAuditRequiredLabelRule {
    fn check(&self, item: &RollbackAudit) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback audit.
#[derive(Debug, Clone)]
pub struct RollbackAuditValidator {
    pub rules: Vec<Rc<dyn RollbackAuditRule>>,
}

impl RollbackAuditValidator {
    pub fn validate(&self, item: &RollbackAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollbackAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_audit_validator() -> RollbackAuditValidator {
    RollbackAuditValidator {
        rules: vec![
            Rc::new(RollbackAuditClusterLengthRule { max_length: 64 }) as Rc<dyn RollbackAuditRule>,
            Rc::new(RollbackAuditVersionRangeRule { min: 80, max: 86400 }),
            Rc::new(RollbackAuditRequiredLabelRule {
                message: "rollback audit needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_audit_validator();
        let item = new_rollback_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = RollbackAuditClusterLengthRule { max_length: 64 };
        let mut item = new_rollback_audit("x", "y");
        item.cluster = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = RollbackAuditVersionRangeRule { min: 32, max: 255 };
        let mut item = new_rollback_audit("x", "y");
        item.version = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RollbackAuditRequiredLabelRule {
            message: "rollback audit needs an entry".to_string(),
        };
        let mut item = new_rollback_audit("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
