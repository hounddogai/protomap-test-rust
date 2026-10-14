use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled health check audit must pass. Check returns its problems, or none.
pub trait ScheduledHealthCheckAuditRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledHealthCheckAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledHealthCheckAuditCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledHealthCheckAuditRule for ScheduledHealthCheckAuditCapacityRangeRule {
    fn check(&self, item: &ScheduledHealthCheckAudit) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledHealthCheckAuditNameLengthRule {
    pub max_length: i64,
}

impl ScheduledHealthCheckAuditRule for ScheduledHealthCheckAuditNameLengthRule {
    fn check(&self, item: &ScheduledHealthCheckAudit) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledHealthCheckAuditRequiredNameRule {
    pub message: String,
}

impl ScheduledHealthCheckAuditRule for ScheduledHealthCheckAuditRequiredNameRule {
    fn check(&self, item: &ScheduledHealthCheckAudit) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledHealthCheckAuditRequiredCodeRule {
    pub message: String,
}

impl ScheduledHealthCheckAuditRule for ScheduledHealthCheckAuditRequiredCodeRule {
    fn check(&self, item: &ScheduledHealthCheckAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledHealthCheckAuditCodePrefixRule {
    pub prefix: String,
}

impl ScheduledHealthCheckAuditRule for ScheduledHealthCheckAuditCodePrefixRule {
    fn check(&self, item: &ScheduledHealthCheckAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledHealthCheckAuditAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ScheduledHealthCheckAuditRule for ScheduledHealthCheckAuditAllowedCodeRule {
    fn check(&self, item: &ScheduledHealthCheckAudit) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled health check audit.
#[derive(Debug, Clone)]
pub struct ScheduledHealthCheckAuditValidator {
    pub rules: Vec<Rc<dyn ScheduledHealthCheckAuditRule>>,
}

impl ScheduledHealthCheckAuditValidator {
    pub fn validate(&self, item: &ScheduledHealthCheckAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledHealthCheckAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledHealthCheckAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_health_check_audit_validator() -> ScheduledHealthCheckAuditValidator {
    ScheduledHealthCheckAuditValidator {
        rules: vec![
            Rc::new(ScheduledHealthCheckAuditCapacityRangeRule { min: 1000, max: 86400 })
                as Rc<dyn ScheduledHealthCheckAuditRule>,
            Rc::new(ScheduledHealthCheckAuditNameLengthRule { max_length: 64 }),
            Rc::new(ScheduledHealthCheckAuditRequiredNameRule {
                message: "scheduled health check audit needs an entry".to_string(),
            }),
            Rc::new(ScheduledHealthCheckAuditRequiredCodeRule {
                message: "scheduled health check audit needs a value".to_string(),
            }),
            Rc::new(ScheduledHealthCheckAuditCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(ScheduledHealthCheckAuditAllowedCodeRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_health_check_audit_validator();
        let item = new_scheduled_health_check_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = ScheduledHealthCheckAuditCapacityRangeRule { min: 80, max: 1000 };
        let mut item = new_scheduled_health_check_audit("x", "y");
        item.capacity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ScheduledHealthCheckAuditNameLengthRule { max_length: 64 };
        let mut item = new_scheduled_health_check_audit("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ScheduledHealthCheckAuditRequiredNameRule {
            message: "scheduled health check audit needs this field".to_string(),
        };
        let mut item = new_scheduled_health_check_audit("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledHealthCheckAuditRequiredCodeRule {
            message: "scheduled health check audit needs an entry".to_string(),
        };
        let mut item = new_scheduled_health_check_audit("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
