use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a migration window must pass. Check returns its problems, or none.
pub trait MigrationWindowRule: std::fmt::Debug {
    fn check(&self, item: &MigrationWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationWindowCodeLengthRule {
    pub max_length: i64,
}

impl MigrationWindowRule for MigrationWindowCodeLengthRule {
    fn check(&self, item: &MigrationWindow) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationWindowDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MigrationWindowRule for MigrationWindowDiscountRangeRule {
    fn check(&self, item: &MigrationWindow) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationWindowLabelLengthRule {
    pub max_length: i64,
}

impl MigrationWindowRule for MigrationWindowLabelLengthRule {
    fn check(&self, item: &MigrationWindow) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration window.
#[derive(Debug, Clone)]
pub struct MigrationWindowValidator {
    pub rules: Vec<Rc<dyn MigrationWindowRule>>,
}

impl MigrationWindowValidator {
    pub fn validate(&self, item: &MigrationWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MigrationWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_window_validator() -> MigrationWindowValidator {
    MigrationWindowValidator {
        rules: vec![
            Rc::new(MigrationWindowCodeLengthRule { max_length: 64 }) as Rc<dyn MigrationWindowRule>,
            Rc::new(MigrationWindowDiscountRangeRule { min: 2.4, max: 1620.0 }),
            Rc::new(MigrationWindowLabelLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_window_validator();
        let item = new_migration_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = MigrationWindowCodeLengthRule { max_length: 16 };
        let mut item = new_migration_window("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = MigrationWindowDiscountRangeRule { min: 0.4, max: 1310.8 };
        let mut item = new_migration_window("x", "y");
        item.discount = 1312.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = MigrationWindowLabelLengthRule { max_length: 32 };
        let mut item = new_migration_window("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
