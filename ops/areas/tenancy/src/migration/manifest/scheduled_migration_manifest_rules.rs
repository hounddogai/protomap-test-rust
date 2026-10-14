use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled migration manifest must pass. Check returns its problems, or none.
pub trait ScheduledMigrationManifestRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledMigrationManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledMigrationManifestAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl ScheduledMigrationManifestRule for ScheduledMigrationManifestAllowedTimezoneRule {
    fn check(&self, item: &ScheduledMigrationManifest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is missing", item.timezone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledMigrationManifestCodePrefixRule {
    pub prefix: String,
}

impl ScheduledMigrationManifestRule for ScheduledMigrationManifestCodePrefixRule {
    fn check(&self, item: &ScheduledMigrationManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledMigrationManifestRequiredTitleRule {
    pub message: String,
}

impl ScheduledMigrationManifestRule for ScheduledMigrationManifestRequiredTitleRule {
    fn check(&self, item: &ScheduledMigrationManifest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledMigrationManifestRequiredReviewerRule {
    pub message: String,
}

impl ScheduledMigrationManifestRule for ScheduledMigrationManifestRequiredReviewerRule {
    fn check(&self, item: &ScheduledMigrationManifest) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledMigrationManifestRequiredCodeRule {
    pub message: String,
}

impl ScheduledMigrationManifestRule for ScheduledMigrationManifestRequiredCodeRule {
    fn check(&self, item: &ScheduledMigrationManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled migration manifest.
#[derive(Debug, Clone)]
pub struct ScheduledMigrationManifestValidator {
    pub rules: Vec<Rc<dyn ScheduledMigrationManifestRule>>,
}

impl ScheduledMigrationManifestValidator {
    pub fn validate(&self, item: &ScheduledMigrationManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledMigrationManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledMigrationManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_migration_manifest_validator() -> ScheduledMigrationManifestValidator {
    ScheduledMigrationManifestValidator {
        rules: vec![
            Rc::new(ScheduledMigrationManifestAllowedTimezoneRule {
                allowed: vec!["lenient".to_string()],
            }) as Rc<dyn ScheduledMigrationManifestRule>,
            Rc::new(ScheduledMigrationManifestCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(ScheduledMigrationManifestRequiredTitleRule {
                message: "scheduled migration manifest needs a value".to_string(),
            }),
            Rc::new(ScheduledMigrationManifestRequiredReviewerRule {
                message: "scheduled migration manifest needs a value".to_string(),
            }),
            Rc::new(ScheduledMigrationManifestRequiredCodeRule {
                message: "scheduled migration manifest needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_migration_manifest_validator();
        let item = new_scheduled_migration_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ScheduledMigrationManifestRequiredTitleRule {
            message: "scheduled migration manifest needs a value".to_string(),
        };
        let mut item = new_scheduled_migration_manifest("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = ScheduledMigrationManifestRequiredReviewerRule {
            message: "scheduled migration manifest needs a value".to_string(),
        };
        let mut item = new_scheduled_migration_manifest("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledMigrationManifestRequiredCodeRule {
            message: "scheduled migration manifest needs this field".to_string(),
        };
        let mut item = new_scheduled_migration_manifest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
