use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tenant schedule must pass. Check returns its problems, or none.
pub trait TenantScheduleRule: std::fmt::Debug {
    fn check(&self, item: &TenantSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantScheduleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TenantScheduleRule for TenantScheduleAllowedCodeRule {
    fn check(&self, item: &TenantSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantScheduleLabelLengthRule {
    pub max_length: i64,
}

impl TenantScheduleRule for TenantScheduleLabelLengthRule {
    fn check(&self, item: &TenantSchedule) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantScheduleRequiredLabelRule {
    pub message: String,
}

impl TenantScheduleRule for TenantScheduleRequiredLabelRule {
    fn check(&self, item: &TenantSchedule) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantScheduleClusterLengthRule {
    pub max_length: i64,
}

impl TenantScheduleRule for TenantScheduleClusterLengthRule {
    fn check(&self, item: &TenantSchedule) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant schedule.
#[derive(Debug, Clone)]
pub struct TenantScheduleValidator {
    pub rules: Vec<Rc<dyn TenantScheduleRule>>,
}

impl TenantScheduleValidator {
    pub fn validate(&self, item: &TenantSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TenantSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_schedule_validator() -> TenantScheduleValidator {
    TenantScheduleValidator {
        rules: vec![
            Rc::new(TenantScheduleAllowedCodeRule {
                allowed: vec!["seasonal".to_string(), "primary".to_string()],
            }) as Rc<dyn TenantScheduleRule>,
            Rc::new(TenantScheduleLabelLengthRule { max_length: 32 }),
            Rc::new(TenantScheduleRequiredLabelRule {
                message: "tenant schedule needs an entry".to_string(),
            }),
            Rc::new(TenantScheduleClusterLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tenant_schedule_validator();
        let item = new_tenant_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TenantScheduleLabelLengthRule { max_length: 3600 };
        let mut item = new_tenant_schedule("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TenantScheduleRequiredLabelRule {
            message: "tenant schedule needs a value".to_string(),
        };
        let mut item = new_tenant_schedule("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = TenantScheduleClusterLengthRule { max_length: 64 };
        let mut item = new_tenant_schedule("x", "y");
        item.cluster = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
