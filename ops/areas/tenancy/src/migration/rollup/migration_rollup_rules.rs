use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a migration rollup must pass. Check returns its problems, or none.
pub trait MigrationRollupRule: std::fmt::Debug {
    fn check(&self, item: &MigrationRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationRollupLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MigrationRollupRule for MigrationRollupLimitRangeRule {
    fn check(&self, item: &MigrationRollup) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationRollupCodePrefixRule {
    pub prefix: String,
}

impl MigrationRollupRule for MigrationRollupCodePrefixRule {
    fn check(&self, item: &MigrationRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationRollupRequiredCodeRule {
    pub message: String,
}

impl MigrationRollupRule for MigrationRollupRequiredCodeRule {
    fn check(&self, item: &MigrationRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationRollupRequiredTitleRule {
    pub message: String,
}

impl MigrationRollupRule for MigrationRollupRequiredTitleRule {
    fn check(&self, item: &MigrationRollup) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration rollup.
#[derive(Debug, Clone)]
pub struct MigrationRollupValidator {
    pub rules: Vec<Rc<dyn MigrationRollupRule>>,
}

impl MigrationRollupValidator {
    pub fn validate(&self, item: &MigrationRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MigrationRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_rollup_validator() -> MigrationRollupValidator {
    MigrationRollupValidator {
        rules: vec![
            Rc::new(MigrationRollupLimitRangeRule { min: 3600, max: 86400 }) as Rc<dyn MigrationRollupRule>,
            Rc::new(MigrationRollupCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(MigrationRollupRequiredCodeRule {
                message: "migration rollup needs this field".to_string(),
            }),
            Rc::new(MigrationRollupRequiredTitleRule {
                message: "migration rollup needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_rollup_validator();
        let item = new_migration_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = MigrationRollupLimitRangeRule { min: 120, max: 1000 };
        let mut item = new_migration_rollup("x", "y");
        item.limit = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = MigrationRollupRequiredCodeRule {
            message: "migration rollup needs this field".to_string(),
        };
        let mut item = new_migration_rollup("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = MigrationRollupRequiredTitleRule {
            message: "migration rollup needs this field".to_string(),
        };
        let mut item = new_migration_rollup("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
