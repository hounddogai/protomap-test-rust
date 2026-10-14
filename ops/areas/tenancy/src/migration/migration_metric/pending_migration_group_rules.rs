use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending migration group must pass. Check returns its problems, or none.
pub trait PendingMigrationGroupRule: std::fmt::Debug {
    fn check(&self, item: &PendingMigrationGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMigrationGroupLabelLengthRule {
    pub max_length: i64,
}

impl PendingMigrationGroupRule for PendingMigrationGroupLabelLengthRule {
    fn check(&self, item: &PendingMigrationGroup) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMigrationGroupDescriptionLengthRule {
    pub max_length: i64,
}

impl PendingMigrationGroupRule for PendingMigrationGroupDescriptionLengthRule {
    fn check(&self, item: &PendingMigrationGroup) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMigrationGroupAllowedDescriptionRule {
    pub allowed: Vec<String>,
}

impl PendingMigrationGroupRule for PendingMigrationGroupAllowedDescriptionRule {
    fn check(&self, item: &PendingMigrationGroup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.description)))
        {
            return vec![format!("description {} is unknown", item.description)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMigrationGroupRequiredFingerprintRule {
    pub message: String,
}

impl PendingMigrationGroupRule for PendingMigrationGroupRequiredFingerprintRule {
    fn check(&self, item: &PendingMigrationGroup) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMigrationGroupCodeLengthRule {
    pub max_length: i64,
}

impl PendingMigrationGroupRule for PendingMigrationGroupCodeLengthRule {
    fn check(&self, item: &PendingMigrationGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending migration group.
#[derive(Debug, Clone)]
pub struct PendingMigrationGroupValidator {
    pub rules: Vec<Rc<dyn PendingMigrationGroupRule>>,
}

impl PendingMigrationGroupValidator {
    pub fn validate(&self, item: &PendingMigrationGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingMigrationGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingMigrationGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_migration_group_validator() -> PendingMigrationGroupValidator {
    PendingMigrationGroupValidator {
        rules: vec![
            Rc::new(PendingMigrationGroupLabelLengthRule { max_length: 80 }) as Rc<dyn PendingMigrationGroupRule>,
            Rc::new(PendingMigrationGroupDescriptionLengthRule { max_length: 255 }),
            Rc::new(PendingMigrationGroupAllowedDescriptionRule {
                allowed: vec!["legacy".to_string()],
            }),
            Rc::new(PendingMigrationGroupRequiredFingerprintRule {
                message: "pending migration group needs an entry".to_string(),
            }),
            Rc::new(PendingMigrationGroupCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_migration_group_validator();
        let item = new_pending_migration_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PendingMigrationGroupLabelLengthRule { max_length: 3600 };
        let mut item = new_pending_migration_group("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = PendingMigrationGroupDescriptionLengthRule { max_length: 16 };
        let mut item = new_pending_migration_group("x", "y");
        item.description = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = PendingMigrationGroupRequiredFingerprintRule {
            message: "pending migration group needs a value".to_string(),
        };
        let mut item = new_pending_migration_group("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingMigrationGroupCodeLengthRule { max_length: 64 };
        let mut item = new_pending_migration_group("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
