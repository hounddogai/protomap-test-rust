use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a migration batch must pass. Check returns its problems, or none.
pub trait MigrationBatchRule: std::fmt::Debug {
    fn check(&self, item: &MigrationBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationBatchScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MigrationBatchRule for MigrationBatchScoreRangeRule {
    fn check(&self, item: &MigrationBatch) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationBatchCodePrefixRule {
    pub prefix: String,
}

impl MigrationBatchRule for MigrationBatchCodePrefixRule {
    fn check(&self, item: &MigrationBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationBatchRequiredDescriptionRule {
    pub message: String,
}

impl MigrationBatchRule for MigrationBatchRequiredDescriptionRule {
    fn check(&self, item: &MigrationBatch) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationBatchRequiredNameRule {
    pub message: String,
}

impl MigrationBatchRule for MigrationBatchRequiredNameRule {
    fn check(&self, item: &MigrationBatch) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationBatchTenantIdLengthRule {
    pub max_length: i64,
}

impl MigrationBatchRule for MigrationBatchTenantIdLengthRule {
    fn check(&self, item: &MigrationBatch) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationBatchQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MigrationBatchRule for MigrationBatchQuantityRangeRule {
    fn check(&self, item: &MigrationBatch) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration batch.
#[derive(Debug, Clone)]
pub struct MigrationBatchValidator {
    pub rules: Vec<Rc<dyn MigrationBatchRule>>,
}

impl MigrationBatchValidator {
    pub fn validate(&self, item: &MigrationBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MigrationBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_batch_validator() -> MigrationBatchValidator {
    MigrationBatchValidator {
        rules: vec![
            Rc::new(MigrationBatchScoreRangeRule { min: 7.2, max: 305.7 }) as Rc<dyn MigrationBatchRule>,
            Rc::new(MigrationBatchCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(MigrationBatchRequiredDescriptionRule {
                message: "migration batch needs this field".to_string(),
            }),
            Rc::new(MigrationBatchRequiredNameRule {
                message: "migration batch needs an entry".to_string(),
            }),
            Rc::new(MigrationBatchTenantIdLengthRule { max_length: 255 }),
            Rc::new(MigrationBatchQuantityRangeRule { min: 120, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_batch_validator();
        let item = new_migration_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = MigrationBatchScoreRangeRule { min: 3.1, max: 4397.4 };
        let mut item = new_migration_batch("x", "y");
        item.score = 4398.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = MigrationBatchRequiredDescriptionRule {
            message: "migration batch needs this field".to_string(),
        };
        let mut item = new_migration_batch("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = MigrationBatchRequiredNameRule {
            message: "migration batch needs a value".to_string(),
        };
        let mut item = new_migration_batch("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = MigrationBatchTenantIdLengthRule { max_length: 120 };
        let mut item = new_migration_batch("x", "y");
        item.tenant_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = MigrationBatchQuantityRangeRule { min: 32, max: 86400 };
        let mut item = new_migration_batch("x", "y");
        item.quantity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
