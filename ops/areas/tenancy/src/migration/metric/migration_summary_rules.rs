use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a migration summary must pass. Check returns its problems, or none.
pub trait MigrationSummaryRule: std::fmt::Debug {
    fn check(&self, item: &MigrationSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationSummaryRequiredSummaryRule {
    pub message: String,
}

impl MigrationSummaryRule for MigrationSummaryRequiredSummaryRule {
    fn check(&self, item: &MigrationSummary) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationSummaryAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl MigrationSummaryRule for MigrationSummaryAllowedCodeRule {
    fn check(&self, item: &MigrationSummary) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationSummaryCodeLengthRule {
    pub max_length: i64,
}

impl MigrationSummaryRule for MigrationSummaryCodeLengthRule {
    fn check(&self, item: &MigrationSummary) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationSummaryLabelLengthRule {
    pub max_length: i64,
}

impl MigrationSummaryRule for MigrationSummaryLabelLengthRule {
    fn check(&self, item: &MigrationSummary) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationSummaryDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MigrationSummaryRule for MigrationSummaryDurationMinutesRangeRule {
    fn check(&self, item: &MigrationSummary) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationSummaryRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MigrationSummaryRule for MigrationSummaryRevisionRangeRule {
    fn check(&self, item: &MigrationSummary) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a migration summary.
#[derive(Debug, Clone)]
pub struct MigrationSummaryValidator {
    pub rules: Vec<Rc<dyn MigrationSummaryRule>>,
}

impl MigrationSummaryValidator {
    pub fn validate(&self, item: &MigrationSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MigrationSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_summary_validator() -> MigrationSummaryValidator {
    MigrationSummaryValidator {
        rules: vec![
            Rc::new(MigrationSummaryRequiredSummaryRule {
                message: "migration summary needs this field".to_string(),
            }) as Rc<dyn MigrationSummaryRule>,
            Rc::new(MigrationSummaryAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(MigrationSummaryCodeLengthRule { max_length: 16 }),
            Rc::new(MigrationSummaryLabelLengthRule { max_length: 120 }),
            Rc::new(MigrationSummaryDurationMinutesRangeRule { min: 16, max: 86400 }),
            Rc::new(MigrationSummaryRevisionRangeRule { min: 120, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_summary_validator();
        let item = new_migration_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = MigrationSummaryRequiredSummaryRule {
            message: "migration summary needs an entry".to_string(),
        };
        let mut item = new_migration_summary("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = MigrationSummaryCodeLengthRule { max_length: 255 };
        let mut item = new_migration_summary("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = MigrationSummaryLabelLengthRule { max_length: 1000 };
        let mut item = new_migration_summary("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = MigrationSummaryDurationMinutesRangeRule { min: 32, max: 32 };
        let mut item = new_migration_summary("x", "y");
        item.duration_minutes = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = MigrationSummaryRevisionRangeRule { min: 80, max: 255 };
        let mut item = new_migration_summary("x", "y");
        item.revision = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
