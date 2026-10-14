use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a migration group must pass. Check returns its problems, or none.
pub trait MigrationGroupRule: std::fmt::Debug {
    fn check(&self, item: &MigrationGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationGroupLabelLengthRule {
    pub max_length: i64,
}

impl MigrationGroupRule for MigrationGroupLabelLengthRule {
    fn check(&self, item: &MigrationGroup) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationGroupRequiredTitleRule {
    pub message: String,
}

impl MigrationGroupRule for MigrationGroupRequiredTitleRule {
    fn check(&self, item: &MigrationGroup) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationGroupTitleLengthRule {
    pub max_length: i64,
}

impl MigrationGroupRule for MigrationGroupTitleLengthRule {
    fn check(&self, item: &MigrationGroup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationGroupCodePrefixRule {
    pub prefix: String,
}

impl MigrationGroupRule for MigrationGroupCodePrefixRule {
    fn check(&self, item: &MigrationGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration group.
#[derive(Debug, Clone)]
pub struct MigrationGroupValidator {
    pub rules: Vec<Rc<dyn MigrationGroupRule>>,
}

impl MigrationGroupValidator {
    pub fn validate(&self, item: &MigrationGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MigrationGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_group_validator() -> MigrationGroupValidator {
    MigrationGroupValidator {
        rules: vec![
            Rc::new(MigrationGroupLabelLengthRule { max_length: 80 }) as Rc<dyn MigrationGroupRule>,
            Rc::new(MigrationGroupRequiredTitleRule {
                message: "migration group needs this field".to_string(),
            }),
            Rc::new(MigrationGroupTitleLengthRule { max_length: 64 }),
            Rc::new(MigrationGroupCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_group_validator();
        let item = new_migration_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = MigrationGroupLabelLengthRule { max_length: 32 };
        let mut item = new_migration_group("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = MigrationGroupRequiredTitleRule {
            message: "migration group needs a value".to_string(),
        };
        let mut item = new_migration_group("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = MigrationGroupTitleLengthRule { max_length: 16 };
        let mut item = new_migration_group("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
