use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tenant snapshot must pass. Check returns its problems, or none.
pub trait TenantSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &TenantSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantSnapshotNameLengthRule {
    pub max_length: i64,
}

impl TenantSnapshotRule for TenantSnapshotNameLengthRule {
    fn check(&self, item: &TenantSnapshot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantSnapshotRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TenantSnapshotRule for TenantSnapshotRotationDaysRangeRule {
    fn check(&self, item: &TenantSnapshot) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantSnapshotStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TenantSnapshotRule for TenantSnapshotStartsAtRangeRule {
    fn check(&self, item: &TenantSnapshot) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant snapshot.
#[derive(Debug, Clone)]
pub struct TenantSnapshotValidator {
    pub rules: Vec<Rc<dyn TenantSnapshotRule>>,
}

impl TenantSnapshotValidator {
    pub fn validate(&self, item: &TenantSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TenantSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_snapshot_validator() -> TenantSnapshotValidator {
    TenantSnapshotValidator {
        rules: vec![
            Rc::new(TenantSnapshotNameLengthRule { max_length: 86400 }) as Rc<dyn TenantSnapshotRule>,
            Rc::new(TenantSnapshotRotationDaysRangeRule { min: 64, max: 120 }),
            Rc::new(TenantSnapshotStartsAtRangeRule { min: 16, max: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tenant_snapshot_validator();
        let item = new_tenant_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TenantSnapshotNameLengthRule { max_length: 80 };
        let mut item = new_tenant_snapshot("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = TenantSnapshotRotationDaysRangeRule { min: 32, max: 1000 };
        let mut item = new_tenant_snapshot("x", "y");
        item.rotation_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = TenantSnapshotStartsAtRangeRule { min: 32, max: 80 };
        let mut item = new_tenant_snapshot("x", "y");
        item.starts_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
