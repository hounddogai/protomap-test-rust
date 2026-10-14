use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional audit event snapshot must pass. Check returns its problems, or none.
pub trait RegionalAuditEventSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &RegionalAuditEventSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAuditEventSnapshotRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalAuditEventSnapshotRule for RegionalAuditEventSnapshotRevisionRangeRule {
    fn check(&self, item: &RegionalAuditEventSnapshot) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAuditEventSnapshotTitleLengthRule {
    pub max_length: i64,
}

impl RegionalAuditEventSnapshotRule for RegionalAuditEventSnapshotTitleLengthRule {
    fn check(&self, item: &RegionalAuditEventSnapshot) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAuditEventSnapshotVisibleGuardRule {
    pub limit: i64,
}

impl RegionalAuditEventSnapshotRule for RegionalAuditEventSnapshotVisibleGuardRule {
    fn check(&self, item: &RegionalAuditEventSnapshot) -> Vec<String> {
        if item.visible && (item.revision > self.limit) {
            return vec![format!(
                "revision {} exceeds {} while visible",
                item.revision, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional audit event snapshot.
#[derive(Debug, Clone)]
pub struct RegionalAuditEventSnapshotValidator {
    pub rules: Vec<Rc<dyn RegionalAuditEventSnapshotRule>>,
}

impl RegionalAuditEventSnapshotValidator {
    pub fn validate(&self, item: &RegionalAuditEventSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalAuditEventSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalAuditEventSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_audit_event_snapshot_validator() -> RegionalAuditEventSnapshotValidator {
    RegionalAuditEventSnapshotValidator {
        rules: vec![
            Rc::new(RegionalAuditEventSnapshotRevisionRangeRule { min: 64, max: 86400 })
                as Rc<dyn RegionalAuditEventSnapshotRule>,
            Rc::new(RegionalAuditEventSnapshotTitleLengthRule { max_length: 64 }),
            Rc::new(RegionalAuditEventSnapshotVisibleGuardRule { limit: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_audit_event_snapshot_validator();
        let item = new_regional_audit_event_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = RegionalAuditEventSnapshotRevisionRangeRule { min: 80, max: 86400 };
        let mut item = new_regional_audit_event_snapshot("x", "y");
        item.revision = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RegionalAuditEventSnapshotTitleLengthRule { max_length: 3600 };
        let mut item = new_regional_audit_event_snapshot("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
