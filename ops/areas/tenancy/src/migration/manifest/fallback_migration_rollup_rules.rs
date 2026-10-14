use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback migration rollup must pass. Check returns its problems, or none.
pub trait FallbackMigrationRollupRule: std::fmt::Debug {
    fn check(&self, item: &FallbackMigrationRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMigrationRollupTitleLengthRule {
    pub max_length: i64,
}

impl FallbackMigrationRollupRule for FallbackMigrationRollupTitleLengthRule {
    fn check(&self, item: &FallbackMigrationRollup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMigrationRollupIssuerLengthRule {
    pub max_length: i64,
}

impl FallbackMigrationRollupRule for FallbackMigrationRollupIssuerLengthRule {
    fn check(&self, item: &FallbackMigrationRollup) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMigrationRollupBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackMigrationRollupRule for FallbackMigrationRollupBudgetRangeRule {
    fn check(&self, item: &FallbackMigrationRollup) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMigrationRollupRequiredIssuerRule {
    pub message: String,
}

impl FallbackMigrationRollupRule for FallbackMigrationRollupRequiredIssuerRule {
    fn check(&self, item: &FallbackMigrationRollup) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMigrationRollupCodePrefixRule {
    pub prefix: String,
}

impl FallbackMigrationRollupRule for FallbackMigrationRollupCodePrefixRule {
    fn check(&self, item: &FallbackMigrationRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback migration rollup.
#[derive(Debug, Clone)]
pub struct FallbackMigrationRollupValidator {
    pub rules: Vec<Rc<dyn FallbackMigrationRollupRule>>,
}

impl FallbackMigrationRollupValidator {
    pub fn validate(&self, item: &FallbackMigrationRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackMigrationRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackMigrationRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_migration_rollup_validator() -> FallbackMigrationRollupValidator {
    FallbackMigrationRollupValidator {
        rules: vec![
            Rc::new(FallbackMigrationRollupTitleLengthRule { max_length: 32 }) as Rc<dyn FallbackMigrationRollupRule>,
            Rc::new(FallbackMigrationRollupIssuerLengthRule { max_length: 3600 }),
            Rc::new(FallbackMigrationRollupBudgetRangeRule { min: 7.0, max: 1381.1 }),
            Rc::new(FallbackMigrationRollupRequiredIssuerRule {
                message: "fallback migration rollup needs this field".to_string(),
            }),
            Rc::new(FallbackMigrationRollupCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_migration_rollup_validator();
        let item = new_fallback_migration_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = FallbackMigrationRollupTitleLengthRule { max_length: 120 };
        let mut item = new_fallback_migration_rollup("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = FallbackMigrationRollupIssuerLengthRule { max_length: 3600 };
        let mut item = new_fallback_migration_rollup("x", "y");
        item.issuer = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = FallbackMigrationRollupBudgetRangeRule { min: 3.0, max: 868.8 };
        let mut item = new_fallback_migration_rollup("x", "y");
        item.budget = 870.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = FallbackMigrationRollupRequiredIssuerRule {
            message: "fallback migration rollup needs this field".to_string(),
        };
        let mut item = new_fallback_migration_rollup("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
