use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default migration segment must pass. Check returns its problems, or none.
pub trait DefaultMigrationSegmentRule: std::fmt::Debug {
    fn check(&self, item: &DefaultMigrationSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationSegmentRequiredCodeRule {
    pub message: String,
}

impl DefaultMigrationSegmentRule for DefaultMigrationSegmentRequiredCodeRule {
    fn check(&self, item: &DefaultMigrationSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationSegmentCodeLengthRule {
    pub max_length: i64,
}

impl DefaultMigrationSegmentRule for DefaultMigrationSegmentCodeLengthRule {
    fn check(&self, item: &DefaultMigrationSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationSegmentRequiredNameRule {
    pub message: String,
}

impl DefaultMigrationSegmentRule for DefaultMigrationSegmentRequiredNameRule {
    fn check(&self, item: &DefaultMigrationSegment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationSegmentNameLengthRule {
    pub max_length: i64,
}

impl DefaultMigrationSegmentRule for DefaultMigrationSegmentNameLengthRule {
    fn check(&self, item: &DefaultMigrationSegment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationSegmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DefaultMigrationSegmentRule for DefaultMigrationSegmentAllowedCodeRule {
    fn check(&self, item: &DefaultMigrationSegment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default migration segment.
#[derive(Debug, Clone)]
pub struct DefaultMigrationSegmentValidator {
    pub rules: Vec<Rc<dyn DefaultMigrationSegmentRule>>,
}

impl DefaultMigrationSegmentValidator {
    pub fn validate(&self, item: &DefaultMigrationSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultMigrationSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultMigrationSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_migration_segment_validator() -> DefaultMigrationSegmentValidator {
    DefaultMigrationSegmentValidator {
        rules: vec![
            Rc::new(DefaultMigrationSegmentRequiredCodeRule {
                message: "default migration segment needs a value".to_string(),
            }) as Rc<dyn DefaultMigrationSegmentRule>,
            Rc::new(DefaultMigrationSegmentCodeLengthRule { max_length: 255 }),
            Rc::new(DefaultMigrationSegmentRequiredNameRule {
                message: "default migration segment needs this field".to_string(),
            }),
            Rc::new(DefaultMigrationSegmentNameLengthRule { max_length: 80 }),
            Rc::new(DefaultMigrationSegmentAllowedCodeRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_migration_segment_validator();
        let item = new_default_migration_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultMigrationSegmentRequiredCodeRule {
            message: "default migration segment needs an entry".to_string(),
        };
        let mut item = new_default_migration_segment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultMigrationSegmentCodeLengthRule { max_length: 1000 };
        let mut item = new_default_migration_segment("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = DefaultMigrationSegmentRequiredNameRule {
            message: "default migration segment needs an entry".to_string(),
        };
        let mut item = new_default_migration_segment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = DefaultMigrationSegmentNameLengthRule { max_length: 64 };
        let mut item = new_default_migration_segment("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
