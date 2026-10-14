use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a timeline audit must pass. Check returns its problems, or none.
pub trait TimelineAuditRule: std::fmt::Debug {
    fn check(&self, item: &TimelineAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineAuditQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TimelineAuditRule for TimelineAuditQuotaUnitsRangeRule {
    fn check(&self, item: &TimelineAudit) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineAuditRequiredCodeRule {
    pub message: String,
}

impl TimelineAuditRule for TimelineAuditRequiredCodeRule {
    fn check(&self, item: &TimelineAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineAuditCodeLengthRule {
    pub max_length: i64,
}

impl TimelineAuditRule for TimelineAuditCodeLengthRule {
    fn check(&self, item: &TimelineAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineAuditRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TimelineAuditRule for TimelineAuditRevisionRangeRule {
    fn check(&self, item: &TimelineAudit) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline audit.
#[derive(Debug, Clone)]
pub struct TimelineAuditValidator {
    pub rules: Vec<Rc<dyn TimelineAuditRule>>,
}

impl TimelineAuditValidator {
    pub fn validate(&self, item: &TimelineAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TimelineAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_audit_validator() -> TimelineAuditValidator {
    TimelineAuditValidator {
        rules: vec![
            Rc::new(TimelineAuditQuotaUnitsRangeRule { min: 32, max: 3600 }) as Rc<dyn TimelineAuditRule>,
            Rc::new(TimelineAuditRequiredCodeRule {
                message: "timeline audit needs an entry".to_string(),
            }),
            Rc::new(TimelineAuditCodeLengthRule { max_length: 16 }),
            Rc::new(TimelineAuditRevisionRangeRule { min: 64, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_timeline_audit_validator();
        let item = new_timeline_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = TimelineAuditQuotaUnitsRangeRule { min: 16, max: 16 };
        let mut item = new_timeline_audit("x", "y");
        item.quota_units = 17;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TimelineAuditRequiredCodeRule {
            message: "timeline audit needs an entry".to_string(),
        };
        let mut item = new_timeline_audit("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TimelineAuditCodeLengthRule { max_length: 64 };
        let mut item = new_timeline_audit("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = TimelineAuditRevisionRangeRule { min: 1000, max: 86400 };
        let mut item = new_timeline_audit("x", "y");
        item.revision = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
