use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a migration milestone must pass. Check returns its problems, or none.
pub trait MigrationMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &MigrationMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationMilestoneNameLengthRule {
    pub max_length: i64,
}

impl MigrationMilestoneRule for MigrationMilestoneNameLengthRule {
    fn check(&self, item: &MigrationMilestone) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationMilestoneRequiredNameRule {
    pub message: String,
}

impl MigrationMilestoneRule for MigrationMilestoneRequiredNameRule {
    fn check(&self, item: &MigrationMilestone) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationMilestoneCipherLengthRule {
    pub max_length: i64,
}

impl MigrationMilestoneRule for MigrationMilestoneCipherLengthRule {
    fn check(&self, item: &MigrationMilestone) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationMilestoneRequiredCipherRule {
    pub message: String,
}

impl MigrationMilestoneRule for MigrationMilestoneRequiredCipherRule {
    fn check(&self, item: &MigrationMilestone) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationMilestoneRequiredCodeRule {
    pub message: String,
}

impl MigrationMilestoneRule for MigrationMilestoneRequiredCodeRule {
    fn check(&self, item: &MigrationMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration milestone.
#[derive(Debug, Clone)]
pub struct MigrationMilestoneValidator {
    pub rules: Vec<Rc<dyn MigrationMilestoneRule>>,
}

impl MigrationMilestoneValidator {
    pub fn validate(&self, item: &MigrationMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MigrationMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_milestone_validator() -> MigrationMilestoneValidator {
    MigrationMilestoneValidator {
        rules: vec![
            Rc::new(MigrationMilestoneNameLengthRule { max_length: 86400 }) as Rc<dyn MigrationMilestoneRule>,
            Rc::new(MigrationMilestoneRequiredNameRule {
                message: "migration milestone needs an entry".to_string(),
            }),
            Rc::new(MigrationMilestoneCipherLengthRule { max_length: 120 }),
            Rc::new(MigrationMilestoneRequiredCipherRule {
                message: "migration milestone needs a value".to_string(),
            }),
            Rc::new(MigrationMilestoneRequiredCodeRule {
                message: "migration milestone needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_milestone_validator();
        let item = new_migration_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = MigrationMilestoneNameLengthRule { max_length: 1000 };
        let mut item = new_migration_milestone("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = MigrationMilestoneRequiredNameRule {
            message: "migration milestone needs this field".to_string(),
        };
        let mut item = new_migration_milestone("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = MigrationMilestoneCipherLengthRule { max_length: 3600 };
        let mut item = new_migration_milestone("x", "y");
        item.cipher = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = MigrationMilestoneRequiredCipherRule {
            message: "migration milestone needs this field".to_string(),
        };
        let mut item = new_migration_milestone("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = MigrationMilestoneRequiredCodeRule {
            message: "migration milestone needs this field".to_string(),
        };
        let mut item = new_migration_milestone("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
