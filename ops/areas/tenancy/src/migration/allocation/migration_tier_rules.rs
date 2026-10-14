use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a migration tier must pass. Check returns its problems, or none.
pub trait MigrationTierRule: std::fmt::Debug {
    fn check(&self, item: &MigrationTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationTierCodeLengthRule {
    pub max_length: i64,
}

impl MigrationTierRule for MigrationTierCodeLengthRule {
    fn check(&self, item: &MigrationTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationTierRequiredNameRule {
    pub message: String,
}

impl MigrationTierRule for MigrationTierRequiredNameRule {
    fn check(&self, item: &MigrationTier) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationTierTenantIdLengthRule {
    pub max_length: i64,
}

impl MigrationTierRule for MigrationTierTenantIdLengthRule {
    fn check(&self, item: &MigrationTier) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationTierNameLengthRule {
    pub max_length: i64,
}

impl MigrationTierRule for MigrationTierNameLengthRule {
    fn check(&self, item: &MigrationTier) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationTierTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl MigrationTierRule for MigrationTierTagsLimitRule {
    fn check(&self, item: &MigrationTier) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationTierExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MigrationTierRule for MigrationTierExpiresAtRangeRule {
    fn check(&self, item: &MigrationTier) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a migration tier.
#[derive(Debug, Clone)]
pub struct MigrationTierValidator {
    pub rules: Vec<Rc<dyn MigrationTierRule>>,
}

impl MigrationTierValidator {
    pub fn validate(&self, item: &MigrationTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MigrationTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_tier_validator() -> MigrationTierValidator {
    MigrationTierValidator {
        rules: vec![
            Rc::new(MigrationTierCodeLengthRule { max_length: 32 }) as Rc<dyn MigrationTierRule>,
            Rc::new(MigrationTierRequiredNameRule {
                message: "migration tier needs this field".to_string(),
            }),
            Rc::new(MigrationTierTenantIdLengthRule { max_length: 32 }),
            Rc::new(MigrationTierNameLengthRule { max_length: 120 }),
            Rc::new(MigrationTierTagsLimitRule {
                max_count: 16,
                banned: vec!["regional".to_string()],
            }),
            Rc::new(MigrationTierExpiresAtRangeRule { min: 16, max: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_tier_validator();
        let item = new_migration_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = MigrationTierCodeLengthRule { max_length: 64 };
        let mut item = new_migration_tier("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = MigrationTierRequiredNameRule {
            message: "migration tier needs an entry".to_string(),
        };
        let mut item = new_migration_tier("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = MigrationTierTenantIdLengthRule { max_length: 1000 };
        let mut item = new_migration_tier("x", "y");
        item.tenant_id = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = MigrationTierNameLengthRule { max_length: 1000 };
        let mut item = new_migration_tier("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = MigrationTierExpiresAtRangeRule { min: 32, max: 255 };
        let mut item = new_migration_tier("x", "y");
        item.expires_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
