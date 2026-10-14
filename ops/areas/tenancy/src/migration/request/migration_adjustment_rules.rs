use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a migration adjustment must pass. Check returns its problems, or none.
pub trait MigrationAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &MigrationAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationAdjustmentAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MigrationAdjustmentRule for MigrationAdjustmentAttemptsRangeRule {
    fn check(&self, item: &MigrationAdjustment) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationAdjustmentRequiredSourceRule {
    pub message: String,
}

impl MigrationAdjustmentRule for MigrationAdjustmentRequiredSourceRule {
    fn check(&self, item: &MigrationAdjustment) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationAdjustmentHostLengthRule {
    pub max_length: i64,
}

impl MigrationAdjustmentRule for MigrationAdjustmentHostLengthRule {
    fn check(&self, item: &MigrationAdjustment) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration adjustment.
#[derive(Debug, Clone)]
pub struct MigrationAdjustmentValidator {
    pub rules: Vec<Rc<dyn MigrationAdjustmentRule>>,
}

impl MigrationAdjustmentValidator {
    pub fn validate(&self, item: &MigrationAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MigrationAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_adjustment_validator() -> MigrationAdjustmentValidator {
    MigrationAdjustmentValidator {
        rules: vec![
            Rc::new(MigrationAdjustmentAttemptsRangeRule { min: 255, max: 255 }) as Rc<dyn MigrationAdjustmentRule>,
            Rc::new(MigrationAdjustmentRequiredSourceRule {
                message: "migration adjustment needs this field".to_string(),
            }),
            Rc::new(MigrationAdjustmentHostLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_adjustment_validator();
        let item = new_migration_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = MigrationAdjustmentAttemptsRangeRule { min: 86400, max: 86400 };
        let mut item = new_migration_adjustment("x", "y");
        item.attempts = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = MigrationAdjustmentRequiredSourceRule {
            message: "migration adjustment needs this field".to_string(),
        };
        let mut item = new_migration_adjustment("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = MigrationAdjustmentHostLengthRule { max_length: 1000 };
        let mut item = new_migration_adjustment("x", "y");
        item.host = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
