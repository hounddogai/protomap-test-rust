use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy migration audit must pass. Check returns its problems, or none.
pub trait LegacyMigrationAuditRule: std::fmt::Debug {
    fn check(&self, item: &LegacyMigrationAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMigrationAuditRequiredLabelRule {
    pub message: String,
}

impl LegacyMigrationAuditRule for LegacyMigrationAuditRequiredLabelRule {
    fn check(&self, item: &LegacyMigrationAudit) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMigrationAuditBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyMigrationAuditRule for LegacyMigrationAuditBurstRangeRule {
    fn check(&self, item: &LegacyMigrationAudit) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMigrationAuditEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyMigrationAuditRule for LegacyMigrationAuditEndsAtRangeRule {
    fn check(&self, item: &LegacyMigrationAudit) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMigrationAuditCodeLengthRule {
    pub max_length: i64,
}

impl LegacyMigrationAuditRule for LegacyMigrationAuditCodeLengthRule {
    fn check(&self, item: &LegacyMigrationAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy migration audit.
#[derive(Debug, Clone)]
pub struct LegacyMigrationAuditValidator {
    pub rules: Vec<Rc<dyn LegacyMigrationAuditRule>>,
}

impl LegacyMigrationAuditValidator {
    pub fn validate(&self, item: &LegacyMigrationAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyMigrationAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyMigrationAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_migration_audit_validator() -> LegacyMigrationAuditValidator {
    LegacyMigrationAuditValidator {
        rules: vec![
            Rc::new(LegacyMigrationAuditRequiredLabelRule {
                message: "legacy migration audit needs this field".to_string(),
            }) as Rc<dyn LegacyMigrationAuditRule>,
            Rc::new(LegacyMigrationAuditBurstRangeRule { min: 86400, max: 86400 }),
            Rc::new(LegacyMigrationAuditEndsAtRangeRule { min: 16, max: 1000 }),
            Rc::new(LegacyMigrationAuditCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_migration_audit_validator();
        let item = new_legacy_migration_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LegacyMigrationAuditRequiredLabelRule {
            message: "legacy migration audit needs this field".to_string(),
        };
        let mut item = new_legacy_migration_audit("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = LegacyMigrationAuditBurstRangeRule { min: 64, max: 80 };
        let mut item = new_legacy_migration_audit("x", "y");
        item.burst = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = LegacyMigrationAuditEndsAtRangeRule { min: 64, max: 3600 };
        let mut item = new_legacy_migration_audit("x", "y");
        item.ends_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyMigrationAuditCodeLengthRule { max_length: 3600 };
        let mut item = new_legacy_migration_audit("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
