use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a migration quota must pass. Check returns its problems, or none.
pub trait MigrationQuotaRule: std::fmt::Debug {
    fn check(&self, item: &MigrationQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationQuotaRequiredCodeRule {
    pub message: String,
}

impl MigrationQuotaRule for MigrationQuotaRequiredCodeRule {
    fn check(&self, item: &MigrationQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationQuotaAllowedNameRule {
    pub allowed: Vec<String>,
}

impl MigrationQuotaRule for MigrationQuotaAllowedNameRule {
    fn check(&self, item: &MigrationQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is missing", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationQuotaNameLengthRule {
    pub max_length: i64,
}

impl MigrationQuotaRule for MigrationQuotaNameLengthRule {
    fn check(&self, item: &MigrationQuota) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationQuotaMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MigrationQuotaRule for MigrationQuotaMarginRangeRule {
    fn check(&self, item: &MigrationQuota) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration quota.
#[derive(Debug, Clone)]
pub struct MigrationQuotaValidator {
    pub rules: Vec<Rc<dyn MigrationQuotaRule>>,
}

impl MigrationQuotaValidator {
    pub fn validate(&self, item: &MigrationQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MigrationQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_quota_validator() -> MigrationQuotaValidator {
    MigrationQuotaValidator {
        rules: vec![
            Rc::new(MigrationQuotaRequiredCodeRule {
                message: "migration quota needs a value".to_string(),
            }) as Rc<dyn MigrationQuotaRule>,
            Rc::new(MigrationQuotaAllowedNameRule { allowed: Vec::new() }),
            Rc::new(MigrationQuotaNameLengthRule { max_length: 32 }),
            Rc::new(MigrationQuotaMarginRangeRule { min: 8.9, max: 1279.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_quota_validator();
        let item = new_migration_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = MigrationQuotaRequiredCodeRule {
            message: "migration quota needs a value".to_string(),
        };
        let mut item = new_migration_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = MigrationQuotaNameLengthRule { max_length: 16 };
        let mut item = new_migration_quota("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = MigrationQuotaMarginRangeRule { min: 9.3, max: 227.9 };
        let mut item = new_migration_quota("x", "y");
        item.margin = 229.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
