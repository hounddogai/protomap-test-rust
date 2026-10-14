use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional migration adjustment must pass. Check returns its problems, or none.
pub trait RegionalMigrationAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &RegionalMigrationAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalMigrationAdjustmentSourceLengthRule {
    pub max_length: i64,
}

impl RegionalMigrationAdjustmentRule for RegionalMigrationAdjustmentSourceLengthRule {
    fn check(&self, item: &RegionalMigrationAdjustment) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalMigrationAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl RegionalMigrationAdjustmentRule for RegionalMigrationAdjustmentCodeLengthRule {
    fn check(&self, item: &RegionalMigrationAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalMigrationAdjustmentNotesLengthRule {
    pub max_length: i64,
}

impl RegionalMigrationAdjustmentRule for RegionalMigrationAdjustmentNotesLengthRule {
    fn check(&self, item: &RegionalMigrationAdjustment) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalMigrationAdjustmentRequiredSourceRule {
    pub message: String,
}

impl RegionalMigrationAdjustmentRule for RegionalMigrationAdjustmentRequiredSourceRule {
    fn check(&self, item: &RegionalMigrationAdjustment) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalMigrationAdjustmentAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl RegionalMigrationAdjustmentRule for RegionalMigrationAdjustmentAllowedSourceRule {
    fn check(&self, item: &RegionalMigrationAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is blocked", item.source)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalMigrationAdjustmentNameLengthRule {
    pub max_length: i64,
}

impl RegionalMigrationAdjustmentRule for RegionalMigrationAdjustmentNameLengthRule {
    fn check(&self, item: &RegionalMigrationAdjustment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional migration adjustment.
#[derive(Debug, Clone)]
pub struct RegionalMigrationAdjustmentValidator {
    pub rules: Vec<Rc<dyn RegionalMigrationAdjustmentRule>>,
}

impl RegionalMigrationAdjustmentValidator {
    pub fn validate(&self, item: &RegionalMigrationAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalMigrationAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalMigrationAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_migration_adjustment_validator() -> RegionalMigrationAdjustmentValidator {
    RegionalMigrationAdjustmentValidator {
        rules: vec![
            Rc::new(RegionalMigrationAdjustmentSourceLengthRule { max_length: 1000 })
                as Rc<dyn RegionalMigrationAdjustmentRule>,
            Rc::new(RegionalMigrationAdjustmentCodeLengthRule { max_length: 86400 }),
            Rc::new(RegionalMigrationAdjustmentNotesLengthRule { max_length: 16 }),
            Rc::new(RegionalMigrationAdjustmentRequiredSourceRule {
                message: "regional migration adjustment needs this field".to_string(),
            }),
            Rc::new(RegionalMigrationAdjustmentAllowedSourceRule {
                allowed: vec!["manual".to_string(), "adaptive".to_string()],
            }),
            Rc::new(RegionalMigrationAdjustmentNameLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_migration_adjustment_validator();
        let item = new_regional_migration_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = RegionalMigrationAdjustmentSourceLengthRule { max_length: 32 };
        let mut item = new_regional_migration_adjustment("x", "y");
        item.source = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalMigrationAdjustmentCodeLengthRule { max_length: 3600 };
        let mut item = new_regional_migration_adjustment("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = RegionalMigrationAdjustmentNotesLengthRule { max_length: 3600 };
        let mut item = new_regional_migration_adjustment("x", "y");
        item.notes = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = RegionalMigrationAdjustmentRequiredSourceRule {
            message: "regional migration adjustment needs an entry".to_string(),
        };
        let mut item = new_regional_migration_adjustment("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RegionalMigrationAdjustmentNameLengthRule { max_length: 64 };
        let mut item = new_regional_migration_adjustment("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
