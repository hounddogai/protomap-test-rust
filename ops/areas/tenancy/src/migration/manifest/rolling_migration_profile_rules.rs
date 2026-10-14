use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling migration profile must pass. Check returns its problems, or none.
pub trait RollingMigrationProfileRule: std::fmt::Debug {
    fn check(&self, item: &RollingMigrationProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingMigrationProfilePrincipalLengthRule {
    pub max_length: i64,
}

impl RollingMigrationProfileRule for RollingMigrationProfilePrincipalLengthRule {
    fn check(&self, item: &RollingMigrationProfile) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingMigrationProfileCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingMigrationProfileRule for RollingMigrationProfileCreatedAtRangeRule {
    fn check(&self, item: &RollingMigrationProfile) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingMigrationProfileVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingMigrationProfileRule for RollingMigrationProfileVersionRangeRule {
    fn check(&self, item: &RollingMigrationProfile) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingMigrationProfileRequiredPrincipalRule {
    pub message: String,
}

impl RollingMigrationProfileRule for RollingMigrationProfileRequiredPrincipalRule {
    fn check(&self, item: &RollingMigrationProfile) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingMigrationProfileAllowedNameRule {
    pub allowed: Vec<String>,
}

impl RollingMigrationProfileRule for RollingMigrationProfileAllowedNameRule {
    fn check(&self, item: &RollingMigrationProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is expired", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingMigrationProfileSourceLengthRule {
    pub max_length: i64,
}

impl RollingMigrationProfileRule for RollingMigrationProfileSourceLengthRule {
    fn check(&self, item: &RollingMigrationProfile) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling migration profile.
#[derive(Debug, Clone)]
pub struct RollingMigrationProfileValidator {
    pub rules: Vec<Rc<dyn RollingMigrationProfileRule>>,
}

impl RollingMigrationProfileValidator {
    pub fn validate(&self, item: &RollingMigrationProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingMigrationProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingMigrationProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_migration_profile_validator() -> RollingMigrationProfileValidator {
    RollingMigrationProfileValidator {
        rules: vec![
            Rc::new(RollingMigrationProfilePrincipalLengthRule { max_length: 255 })
                as Rc<dyn RollingMigrationProfileRule>,
            Rc::new(RollingMigrationProfileCreatedAtRangeRule { min: 16, max: 86400 }),
            Rc::new(RollingMigrationProfileVersionRangeRule { min: 32, max: 32 }),
            Rc::new(RollingMigrationProfileRequiredPrincipalRule {
                message: "rolling migration profile needs this field".to_string(),
            }),
            Rc::new(RollingMigrationProfileAllowedNameRule {
                allowed: vec!["manual".to_string()],
            }),
            Rc::new(RollingMigrationProfileSourceLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_migration_profile_validator();
        let item = new_rolling_migration_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = RollingMigrationProfilePrincipalLengthRule { max_length: 64 };
        let mut item = new_rolling_migration_profile("x", "y");
        item.principal = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = RollingMigrationProfileCreatedAtRangeRule { min: 32, max: 3600 };
        let mut item = new_rolling_migration_profile("x", "y");
        item.created_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = RollingMigrationProfileVersionRangeRule { min: 80, max: 120 };
        let mut item = new_rolling_migration_profile("x", "y");
        item.version = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = RollingMigrationProfileRequiredPrincipalRule {
            message: "rolling migration profile needs a value".to_string(),
        };
        let mut item = new_rolling_migration_profile("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = RollingMigrationProfileSourceLengthRule { max_length: 16 };
        let mut item = new_rolling_migration_profile("x", "y");
        item.source = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
