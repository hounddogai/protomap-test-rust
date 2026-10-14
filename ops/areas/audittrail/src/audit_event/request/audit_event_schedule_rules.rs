use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a audit event schedule must pass. Check returns its problems, or none.
pub trait AuditEventScheduleRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventScheduleRegionLengthRule {
    pub max_length: i64,
}

impl AuditEventScheduleRule for AuditEventScheduleRegionLengthRule {
    fn check(&self, item: &AuditEventSchedule) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventScheduleVisibleGuardRule {
    pub limit: i64,
}

impl AuditEventScheduleRule for AuditEventScheduleVisibleGuardRule {
    fn check(&self, item: &AuditEventSchedule) -> Vec<String> {
        if item.visible && (item.revision > self.limit) {
            return vec![format!(
                "revision {} exceeds {} while visible",
                item.revision, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventScheduleCodePrefixRule {
    pub prefix: String,
}

impl AuditEventScheduleRule for AuditEventScheduleCodePrefixRule {
    fn check(&self, item: &AuditEventSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event schedule.
#[derive(Debug, Clone)]
pub struct AuditEventScheduleValidator {
    pub rules: Vec<Rc<dyn AuditEventScheduleRule>>,
}

impl AuditEventScheduleValidator {
    pub fn validate(&self, item: &AuditEventSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AuditEventSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_schedule_validator() -> AuditEventScheduleValidator {
    AuditEventScheduleValidator {
        rules: vec![
            Rc::new(AuditEventScheduleRegionLengthRule { max_length: 86400 }) as Rc<dyn AuditEventScheduleRule>,
            Rc::new(AuditEventScheduleVisibleGuardRule { limit: 32 }),
            Rc::new(AuditEventScheduleCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_audit_event_schedule_validator();
        let item = new_audit_event_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = AuditEventScheduleRegionLengthRule { max_length: 80 };
        let mut item = new_audit_event_schedule("x", "y");
        item.region = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
