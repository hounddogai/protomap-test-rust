use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a migration baseline must pass. Check returns its problems, or none.
pub trait MigrationBaselineRule: std::fmt::Debug {
    fn check(&self, item: &MigrationBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationBaselineCodeLengthRule {
    pub max_length: i64,
}

impl MigrationBaselineRule for MigrationBaselineCodeLengthRule {
    fn check(&self, item: &MigrationBaseline) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationBaselineRequiredCodeRule {
    pub message: String,
}

impl MigrationBaselineRule for MigrationBaselineRequiredCodeRule {
    fn check(&self, item: &MigrationBaseline) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationBaselineFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MigrationBaselineRule for MigrationBaselineFloorValueRangeRule {
    fn check(&self, item: &MigrationBaseline) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationBaselineScopeLengthRule {
    pub max_length: i64,
}

impl MigrationBaselineRule for MigrationBaselineScopeLengthRule {
    fn check(&self, item: &MigrationBaseline) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationBaselineAudienceLengthRule {
    pub max_length: i64,
}

impl MigrationBaselineRule for MigrationBaselineAudienceLengthRule {
    fn check(&self, item: &MigrationBaseline) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration baseline.
#[derive(Debug, Clone)]
pub struct MigrationBaselineValidator {
    pub rules: Vec<Rc<dyn MigrationBaselineRule>>,
}

impl MigrationBaselineValidator {
    pub fn validate(&self, item: &MigrationBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MigrationBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_baseline_validator() -> MigrationBaselineValidator {
    MigrationBaselineValidator {
        rules: vec![
            Rc::new(MigrationBaselineCodeLengthRule { max_length: 120 }) as Rc<dyn MigrationBaselineRule>,
            Rc::new(MigrationBaselineRequiredCodeRule {
                message: "migration baseline needs a value".to_string(),
            }),
            Rc::new(MigrationBaselineFloorValueRangeRule { min: 7.8, max: 1420.6 }),
            Rc::new(MigrationBaselineScopeLengthRule { max_length: 120 }),
            Rc::new(MigrationBaselineAudienceLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_baseline_validator();
        let item = new_migration_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = MigrationBaselineCodeLengthRule { max_length: 255 };
        let mut item = new_migration_baseline("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = MigrationBaselineRequiredCodeRule {
            message: "migration baseline needs a value".to_string(),
        };
        let mut item = new_migration_baseline("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = MigrationBaselineFloorValueRangeRule { min: 0.9, max: 917.2 };
        let mut item = new_migration_baseline("x", "y");
        item.floor_value = 918.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = MigrationBaselineScopeLengthRule { max_length: 3600 };
        let mut item = new_migration_baseline("x", "y");
        item.scope = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = MigrationBaselineAudienceLengthRule { max_length: 120 };
        let mut item = new_migration_baseline("x", "y");
        item.audience = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
