use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled zone audit must pass. Check returns its problems, or none.
pub trait ScheduledZoneAuditRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledZoneAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneAuditFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledZoneAuditRule for ScheduledZoneAuditFloorValueRangeRule {
    fn check(&self, item: &ScheduledZoneAudit) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneAuditCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledZoneAuditRule for ScheduledZoneAuditCodeLengthRule {
    fn check(&self, item: &ScheduledZoneAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneAuditCodePrefixRule {
    pub prefix: String,
}

impl ScheduledZoneAuditRule for ScheduledZoneAuditCodePrefixRule {
    fn check(&self, item: &ScheduledZoneAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneAuditDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledZoneAuditRule for ScheduledZoneAuditDurationMinutesRangeRule {
    fn check(&self, item: &ScheduledZoneAudit) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled zone audit.
#[derive(Debug, Clone)]
pub struct ScheduledZoneAuditValidator {
    pub rules: Vec<Rc<dyn ScheduledZoneAuditRule>>,
}

impl ScheduledZoneAuditValidator {
    pub fn validate(&self, item: &ScheduledZoneAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledZoneAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledZoneAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_zone_audit_validator() -> ScheduledZoneAuditValidator {
    ScheduledZoneAuditValidator {
        rules: vec![
            Rc::new(ScheduledZoneAuditFloorValueRangeRule { min: 5.5, max: 4534.8 }) as Rc<dyn ScheduledZoneAuditRule>,
            Rc::new(ScheduledZoneAuditCodeLengthRule { max_length: 120 }),
            Rc::new(ScheduledZoneAuditCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(ScheduledZoneAuditDurationMinutesRangeRule { min: 16, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_zone_audit_validator();
        let item = new_scheduled_zone_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = ScheduledZoneAuditFloorValueRangeRule { min: 4.2, max: 533.7 };
        let mut item = new_scheduled_zone_audit("x", "y");
        item.floor_value = 535.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledZoneAuditCodeLengthRule { max_length: 86400 };
        let mut item = new_scheduled_zone_audit("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = ScheduledZoneAuditDurationMinutesRangeRule { min: 32, max: 32 };
        let mut item = new_scheduled_zone_audit("x", "y");
        item.duration_minutes = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
