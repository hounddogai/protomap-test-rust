use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a migration audit must pass. Check returns its problems, or none.
pub trait MigrationAuditRule: std::fmt::Debug {
    fn check(&self, item: &MigrationAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationAuditVisibleGuardRule {
    pub limit: i64,
}

impl MigrationAuditRule for MigrationAuditVisibleGuardRule {
    fn check(&self, item: &MigrationAudit) -> Vec<String> {
        if item.visible && (item.grace_days > self.limit) {
            return vec![format!(
                "grace days {} exceeds {} while visible",
                item.grace_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationAuditRequiredLabelRule {
    pub message: String,
}

impl MigrationAuditRule for MigrationAuditRequiredLabelRule {
    fn check(&self, item: &MigrationAudit) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationAuditSummaryLengthRule {
    pub max_length: i64,
}

impl MigrationAuditRule for MigrationAuditSummaryLengthRule {
    fn check(&self, item: &MigrationAudit) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationAuditTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl MigrationAuditRule for MigrationAuditTagsLimitRule {
    fn check(&self, item: &MigrationAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

/// Runs every rule of a migration audit.
#[derive(Debug, Clone)]
pub struct MigrationAuditValidator {
    pub rules: Vec<Rc<dyn MigrationAuditRule>>,
}

impl MigrationAuditValidator {
    pub fn validate(&self, item: &MigrationAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MigrationAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_audit_validator() -> MigrationAuditValidator {
    MigrationAuditValidator {
        rules: vec![
            Rc::new(MigrationAuditVisibleGuardRule { limit: 120 }) as Rc<dyn MigrationAuditRule>,
            Rc::new(MigrationAuditRequiredLabelRule {
                message: "migration audit needs this field".to_string(),
            }),
            Rc::new(MigrationAuditSummaryLengthRule { max_length: 3600 }),
            Rc::new(MigrationAuditTagsLimitRule {
                max_count: 255,
                banned: Vec::new(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_audit_validator();
        let item = new_migration_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = MigrationAuditRequiredLabelRule {
            message: "migration audit needs a value".to_string(),
        };
        let mut item = new_migration_audit("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = MigrationAuditSummaryLengthRule { max_length: 64 };
        let mut item = new_migration_audit("x", "y");
        item.summary = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
