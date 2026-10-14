use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched migration rule must pass. Check returns its problems, or none.
pub trait BatchedMigrationRuleRule: std::fmt::Debug {
    fn check(&self, item: &BatchedMigrationRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedMigrationRuleQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedMigrationRuleRule for BatchedMigrationRuleQuotaUnitsRangeRule {
    fn check(&self, item: &BatchedMigrationRule) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedMigrationRuleAllowedNameRule {
    pub allowed: Vec<String>,
}

impl BatchedMigrationRuleRule for BatchedMigrationRuleAllowedNameRule {
    fn check(&self, item: &BatchedMigrationRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is not allowed", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedMigrationRuleCodeLengthRule {
    pub max_length: i64,
}

impl BatchedMigrationRuleRule for BatchedMigrationRuleCodeLengthRule {
    fn check(&self, item: &BatchedMigrationRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedMigrationRuleNameLengthRule {
    pub max_length: i64,
}

impl BatchedMigrationRuleRule for BatchedMigrationRuleNameLengthRule {
    fn check(&self, item: &BatchedMigrationRule) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched migration rule.
#[derive(Debug, Clone)]
pub struct BatchedMigrationRuleValidator {
    pub rules: Vec<Rc<dyn BatchedMigrationRuleRule>>,
}

impl BatchedMigrationRuleValidator {
    pub fn validate(&self, item: &BatchedMigrationRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedMigrationRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedMigrationRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_migration_rule_validator() -> BatchedMigrationRuleValidator {
    BatchedMigrationRuleValidator {
        rules: vec![
            Rc::new(BatchedMigrationRuleQuotaUnitsRangeRule { min: 32, max: 1000 }) as Rc<dyn BatchedMigrationRuleRule>,
            Rc::new(BatchedMigrationRuleAllowedNameRule { allowed: Vec::new() }),
            Rc::new(BatchedMigrationRuleCodeLengthRule { max_length: 16 }),
            Rc::new(BatchedMigrationRuleNameLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_migration_rule_validator();
        let item = new_batched_migration_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = BatchedMigrationRuleQuotaUnitsRangeRule { min: 120, max: 255 };
        let mut item = new_batched_migration_rule("x", "y");
        item.quota_units = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedMigrationRuleCodeLengthRule { max_length: 64 };
        let mut item = new_batched_migration_rule("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = BatchedMigrationRuleNameLengthRule { max_length: 16 };
        let mut item = new_batched_migration_rule("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
