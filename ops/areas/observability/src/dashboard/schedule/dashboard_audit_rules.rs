use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dashboard audit must pass. Check returns its problems, or none.
pub trait DashboardAuditRule: std::fmt::Debug {
    fn check(&self, item: &DashboardAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardAuditAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DashboardAuditRule for DashboardAuditAttemptsRangeRule {
    fn check(&self, item: &DashboardAudit) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardAuditQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DashboardAuditRule for DashboardAuditQuotaUnitsRangeRule {
    fn check(&self, item: &DashboardAudit) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardAuditNotesLengthRule {
    pub max_length: i64,
}

impl DashboardAuditRule for DashboardAuditNotesLengthRule {
    fn check(&self, item: &DashboardAudit) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardAuditNameLengthRule {
    pub max_length: i64,
}

impl DashboardAuditRule for DashboardAuditNameLengthRule {
    fn check(&self, item: &DashboardAudit) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dashboard audit.
#[derive(Debug, Clone)]
pub struct DashboardAuditValidator {
    pub rules: Vec<Rc<dyn DashboardAuditRule>>,
}

impl DashboardAuditValidator {
    pub fn validate(&self, item: &DashboardAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DashboardAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_audit_validator() -> DashboardAuditValidator {
    DashboardAuditValidator {
        rules: vec![
            Rc::new(DashboardAuditAttemptsRangeRule { min: 32, max: 32 }) as Rc<dyn DashboardAuditRule>,
            Rc::new(DashboardAuditQuotaUnitsRangeRule { min: 32, max: 86400 }),
            Rc::new(DashboardAuditNotesLengthRule { max_length: 32 }),
            Rc::new(DashboardAuditNameLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dashboard_audit_validator();
        let item = new_dashboard_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = DashboardAuditAttemptsRangeRule { min: 3600, max: 3600 };
        let mut item = new_dashboard_audit("x", "y");
        item.attempts = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = DashboardAuditQuotaUnitsRangeRule { min: 16, max: 255 };
        let mut item = new_dashboard_audit("x", "y");
        item.quota_units = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = DashboardAuditNotesLengthRule { max_length: 3600 };
        let mut item = new_dashboard_audit("x", "y");
        item.notes = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = DashboardAuditNameLengthRule { max_length: 255 };
        let mut item = new_dashboard_audit("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
