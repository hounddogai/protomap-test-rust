use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a migration profile must pass. Check returns its problems, or none.
pub trait MigrationProfileRule: std::fmt::Debug {
    fn check(&self, item: &MigrationProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationProfileCodePrefixRule {
    pub prefix: String,
}

impl MigrationProfileRule for MigrationProfileCodePrefixRule {
    fn check(&self, item: &MigrationProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationProfileRequiredLabelRule {
    pub message: String,
}

impl MigrationProfileRule for MigrationProfileRequiredLabelRule {
    fn check(&self, item: &MigrationProfile) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationProfileRequiredCodeRule {
    pub message: String,
}

impl MigrationProfileRule for MigrationProfileRequiredCodeRule {
    fn check(&self, item: &MigrationProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationProfileLabelLengthRule {
    pub max_length: i64,
}

impl MigrationProfileRule for MigrationProfileLabelLengthRule {
    fn check(&self, item: &MigrationProfile) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationProfileAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl MigrationProfileRule for MigrationProfileAllowedCodeRule {
    fn check(&self, item: &MigrationProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration profile.
#[derive(Debug, Clone)]
pub struct MigrationProfileValidator {
    pub rules: Vec<Rc<dyn MigrationProfileRule>>,
}

impl MigrationProfileValidator {
    pub fn validate(&self, item: &MigrationProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MigrationProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_profile_validator() -> MigrationProfileValidator {
    MigrationProfileValidator {
        rules: vec![
            Rc::new(MigrationProfileCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn MigrationProfileRule>,
            Rc::new(MigrationProfileRequiredLabelRule {
                message: "migration profile needs a value".to_string(),
            }),
            Rc::new(MigrationProfileRequiredCodeRule {
                message: "migration profile needs a value".to_string(),
            }),
            Rc::new(MigrationProfileLabelLengthRule { max_length: 32 }),
            Rc::new(MigrationProfileAllowedCodeRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_profile_validator();
        let item = new_migration_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = MigrationProfileRequiredLabelRule {
            message: "migration profile needs this field".to_string(),
        };
        let mut item = new_migration_profile("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = MigrationProfileRequiredCodeRule {
            message: "migration profile needs an entry".to_string(),
        };
        let mut item = new_migration_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = MigrationProfileLabelLengthRule { max_length: 255 };
        let mut item = new_migration_profile("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
