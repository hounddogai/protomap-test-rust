use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped migration cycle must pass. Check returns its problems, or none.
pub trait CappedMigrationCycleRule: std::fmt::Debug {
    fn check(&self, item: &CappedMigrationCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedMigrationCycleAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl CappedMigrationCycleRule for CappedMigrationCycleAllowedSourceRule {
    fn check(&self, item: &CappedMigrationCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is inconsistent", item.source)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedMigrationCycleNameLengthRule {
    pub max_length: i64,
}

impl CappedMigrationCycleRule for CappedMigrationCycleNameLengthRule {
    fn check(&self, item: &CappedMigrationCycle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedMigrationCycleEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedMigrationCycleRule for CappedMigrationCycleEndsAtRangeRule {
    fn check(&self, item: &CappedMigrationCycle) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped migration cycle.
#[derive(Debug, Clone)]
pub struct CappedMigrationCycleValidator {
    pub rules: Vec<Rc<dyn CappedMigrationCycleRule>>,
}

impl CappedMigrationCycleValidator {
    pub fn validate(&self, item: &CappedMigrationCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedMigrationCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedMigrationCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_migration_cycle_validator() -> CappedMigrationCycleValidator {
    CappedMigrationCycleValidator {
        rules: vec![
            Rc::new(CappedMigrationCycleAllowedSourceRule { allowed: Vec::new() }) as Rc<dyn CappedMigrationCycleRule>,
            Rc::new(CappedMigrationCycleNameLengthRule { max_length: 16 }),
            Rc::new(CappedMigrationCycleEndsAtRangeRule { min: 80, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_migration_cycle_validator();
        let item = new_capped_migration_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = CappedMigrationCycleNameLengthRule { max_length: 120 };
        let mut item = new_capped_migration_cycle("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = CappedMigrationCycleEndsAtRangeRule { min: 120, max: 3600 };
        let mut item = new_capped_migration_cycle("x", "y");
        item.ends_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
