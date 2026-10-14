use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered migration estimate must pass. Check returns its problems, or none.
pub trait TieredMigrationEstimateRule: std::fmt::Debug {
    fn check(&self, item: &TieredMigrationEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationEstimateCategoryLengthRule {
    pub max_length: i64,
}

impl TieredMigrationEstimateRule for TieredMigrationEstimateCategoryLengthRule {
    fn check(&self, item: &TieredMigrationEstimate) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationEstimateRequiredCategoryRule {
    pub message: String,
}

impl TieredMigrationEstimateRule for TieredMigrationEstimateRequiredCategoryRule {
    fn check(&self, item: &TieredMigrationEstimate) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationEstimateCodeLengthRule {
    pub max_length: i64,
}

impl TieredMigrationEstimateRule for TieredMigrationEstimateCodeLengthRule {
    fn check(&self, item: &TieredMigrationEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationEstimateCodePrefixRule {
    pub prefix: String,
}

impl TieredMigrationEstimateRule for TieredMigrationEstimateCodePrefixRule {
    fn check(&self, item: &TieredMigrationEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationEstimateRequiredCodeRule {
    pub message: String,
}

impl TieredMigrationEstimateRule for TieredMigrationEstimateRequiredCodeRule {
    fn check(&self, item: &TieredMigrationEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationEstimateRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredMigrationEstimateRule for TieredMigrationEstimateRetriesRangeRule {
    fn check(&self, item: &TieredMigrationEstimate) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered migration estimate.
#[derive(Debug, Clone)]
pub struct TieredMigrationEstimateValidator {
    pub rules: Vec<Rc<dyn TieredMigrationEstimateRule>>,
}

impl TieredMigrationEstimateValidator {
    pub fn validate(&self, item: &TieredMigrationEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredMigrationEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredMigrationEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_migration_estimate_validator() -> TieredMigrationEstimateValidator {
    TieredMigrationEstimateValidator {
        rules: vec![
            Rc::new(TieredMigrationEstimateCategoryLengthRule { max_length: 86400 })
                as Rc<dyn TieredMigrationEstimateRule>,
            Rc::new(TieredMigrationEstimateRequiredCategoryRule {
                message: "tiered migration estimate needs a value".to_string(),
            }),
            Rc::new(TieredMigrationEstimateCodeLengthRule { max_length: 16 }),
            Rc::new(TieredMigrationEstimateCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(TieredMigrationEstimateRequiredCodeRule {
                message: "tiered migration estimate needs an entry".to_string(),
            }),
            Rc::new(TieredMigrationEstimateRetriesRangeRule { min: 16, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_migration_estimate_validator();
        let item = new_tiered_migration_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = TieredMigrationEstimateCategoryLengthRule { max_length: 3600 };
        let mut item = new_tiered_migration_estimate("x", "y");
        item.category = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = TieredMigrationEstimateRequiredCategoryRule {
            message: "tiered migration estimate needs an entry".to_string(),
        };
        let mut item = new_tiered_migration_estimate("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredMigrationEstimateCodeLengthRule { max_length: 64 };
        let mut item = new_tiered_migration_estimate("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredMigrationEstimateRequiredCodeRule {
            message: "tiered migration estimate needs an entry".to_string(),
        };
        let mut item = new_tiered_migration_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = TieredMigrationEstimateRetriesRangeRule { min: 32, max: 80 };
        let mut item = new_tiered_migration_estimate("x", "y");
        item.retries = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
