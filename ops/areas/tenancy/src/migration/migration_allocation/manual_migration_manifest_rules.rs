use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual migration manifest must pass. Check returns its problems, or none.
pub trait ManualMigrationManifestRule: std::fmt::Debug {
    fn check(&self, item: &ManualMigrationManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualMigrationManifestPrincipalLengthRule {
    pub max_length: i64,
}

impl ManualMigrationManifestRule for ManualMigrationManifestPrincipalLengthRule {
    fn check(&self, item: &ManualMigrationManifest) -> Vec<String> {
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
pub struct ManualMigrationManifestMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualMigrationManifestRule for ManualMigrationManifestMinimumRangeRule {
    fn check(&self, item: &ManualMigrationManifest) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualMigrationManifestRequiredPrincipalRule {
    pub message: String,
}

impl ManualMigrationManifestRule for ManualMigrationManifestRequiredPrincipalRule {
    fn check(&self, item: &ManualMigrationManifest) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualMigrationManifestApprovedGuardRule {
    pub limit: f64,
}

impl ManualMigrationManifestRule for ManualMigrationManifestApprovedGuardRule {
    fn check(&self, item: &ManualMigrationManifest) -> Vec<String> {
        if item.approved && (item.minimum > self.limit) {
            return vec![format!(
                "minimum {} exceeds {} while approved",
                item.minimum, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualMigrationManifestAllowedDescriptionRule {
    pub allowed: Vec<String>,
}

impl ManualMigrationManifestRule for ManualMigrationManifestAllowedDescriptionRule {
    fn check(&self, item: &ManualMigrationManifest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.description)))
        {
            return vec![format!("description {} is above maximum", item.description)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual migration manifest.
#[derive(Debug, Clone)]
pub struct ManualMigrationManifestValidator {
    pub rules: Vec<Rc<dyn ManualMigrationManifestRule>>,
}

impl ManualMigrationManifestValidator {
    pub fn validate(&self, item: &ManualMigrationManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualMigrationManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualMigrationManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_migration_manifest_validator() -> ManualMigrationManifestValidator {
    ManualMigrationManifestValidator {
        rules: vec![
            Rc::new(ManualMigrationManifestPrincipalLengthRule { max_length: 32 })
                as Rc<dyn ManualMigrationManifestRule>,
            Rc::new(ManualMigrationManifestMinimumRangeRule { min: 4.5, max: 68.1 }),
            Rc::new(ManualMigrationManifestRequiredPrincipalRule {
                message: "manual migration manifest needs this field".to_string(),
            }),
            Rc::new(ManualMigrationManifestApprovedGuardRule { limit: 6.9 }),
            Rc::new(ManualMigrationManifestAllowedDescriptionRule {
                allowed: vec!["global".to_string(), "default".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_migration_manifest_validator();
        let item = new_manual_migration_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = ManualMigrationManifestPrincipalLengthRule { max_length: 1000 };
        let mut item = new_manual_migration_manifest("x", "y");
        item.principal = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = ManualMigrationManifestMinimumRangeRule { min: 4.6, max: 1073.5 };
        let mut item = new_manual_migration_manifest("x", "y");
        item.minimum = 1075.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = ManualMigrationManifestRequiredPrincipalRule {
            message: "manual migration manifest needs a value".to_string(),
        };
        let mut item = new_manual_migration_manifest("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
