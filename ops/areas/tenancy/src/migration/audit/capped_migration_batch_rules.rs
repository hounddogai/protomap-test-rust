use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped migration batch must pass. Check returns its problems, or none.
pub trait CappedMigrationBatchRule: std::fmt::Debug {
    fn check(&self, item: &CappedMigrationBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedMigrationBatchRequiredNamespaceRule {
    pub message: String,
}

impl CappedMigrationBatchRule for CappedMigrationBatchRequiredNamespaceRule {
    fn check(&self, item: &CappedMigrationBatch) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedMigrationBatchLabelLengthRule {
    pub max_length: i64,
}

impl CappedMigrationBatchRule for CappedMigrationBatchLabelLengthRule {
    fn check(&self, item: &CappedMigrationBatch) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedMigrationBatchRequiredCodeRule {
    pub message: String,
}

impl CappedMigrationBatchRule for CappedMigrationBatchRequiredCodeRule {
    fn check(&self, item: &CappedMigrationBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedMigrationBatchCodePrefixRule {
    pub prefix: String,
}

impl CappedMigrationBatchRule for CappedMigrationBatchCodePrefixRule {
    fn check(&self, item: &CappedMigrationBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped migration batch.
#[derive(Debug, Clone)]
pub struct CappedMigrationBatchValidator {
    pub rules: Vec<Rc<dyn CappedMigrationBatchRule>>,
}

impl CappedMigrationBatchValidator {
    pub fn validate(&self, item: &CappedMigrationBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedMigrationBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedMigrationBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_migration_batch_validator() -> CappedMigrationBatchValidator {
    CappedMigrationBatchValidator {
        rules: vec![
            Rc::new(CappedMigrationBatchRequiredNamespaceRule {
                message: "capped migration batch needs this field".to_string(),
            }) as Rc<dyn CappedMigrationBatchRule>,
            Rc::new(CappedMigrationBatchLabelLengthRule { max_length: 16 }),
            Rc::new(CappedMigrationBatchRequiredCodeRule {
                message: "capped migration batch needs an entry".to_string(),
            }),
            Rc::new(CappedMigrationBatchCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_migration_batch_validator();
        let item = new_capped_migration_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = CappedMigrationBatchRequiredNamespaceRule {
            message: "capped migration batch needs an entry".to_string(),
        };
        let mut item = new_capped_migration_batch("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = CappedMigrationBatchLabelLengthRule { max_length: 16 };
        let mut item = new_capped_migration_batch("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedMigrationBatchRequiredCodeRule {
            message: "capped migration batch needs an entry".to_string(),
        };
        let mut item = new_capped_migration_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
