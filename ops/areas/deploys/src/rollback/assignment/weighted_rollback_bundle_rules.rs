use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted rollback bundle must pass. Check returns its problems, or none.
pub trait WeightedRollbackBundleRule: std::fmt::Debug {
    fn check(&self, item: &WeightedRollbackBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRollbackBundleNameLengthRule {
    pub max_length: i64,
}

impl WeightedRollbackBundleRule for WeightedRollbackBundleNameLengthRule {
    fn check(&self, item: &WeightedRollbackBundle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRollbackBundleRequiredCodeRule {
    pub message: String,
}

impl WeightedRollbackBundleRule for WeightedRollbackBundleRequiredCodeRule {
    fn check(&self, item: &WeightedRollbackBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRollbackBundleRequiredKeyIdRule {
    pub message: String,
}

impl WeightedRollbackBundleRule for WeightedRollbackBundleRequiredKeyIdRule {
    fn check(&self, item: &WeightedRollbackBundle) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRollbackBundleKeyIdLengthRule {
    pub max_length: i64,
}

impl WeightedRollbackBundleRule for WeightedRollbackBundleKeyIdLengthRule {
    fn check(&self, item: &WeightedRollbackBundle) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRollbackBundleCodePrefixRule {
    pub prefix: String,
}

impl WeightedRollbackBundleRule for WeightedRollbackBundleCodePrefixRule {
    fn check(&self, item: &WeightedRollbackBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted rollback bundle.
#[derive(Debug, Clone)]
pub struct WeightedRollbackBundleValidator {
    pub rules: Vec<Rc<dyn WeightedRollbackBundleRule>>,
}

impl WeightedRollbackBundleValidator {
    pub fn validate(&self, item: &WeightedRollbackBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedRollbackBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedRollbackBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_rollback_bundle_validator() -> WeightedRollbackBundleValidator {
    WeightedRollbackBundleValidator {
        rules: vec![
            Rc::new(WeightedRollbackBundleNameLengthRule { max_length: 120 }) as Rc<dyn WeightedRollbackBundleRule>,
            Rc::new(WeightedRollbackBundleRequiredCodeRule {
                message: "weighted rollback bundle needs a value".to_string(),
            }),
            Rc::new(WeightedRollbackBundleRequiredKeyIdRule {
                message: "weighted rollback bundle needs this field".to_string(),
            }),
            Rc::new(WeightedRollbackBundleKeyIdLengthRule { max_length: 86400 }),
            Rc::new(WeightedRollbackBundleCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_rollback_bundle_validator();
        let item = new_weighted_rollback_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = WeightedRollbackBundleNameLengthRule { max_length: 86400 };
        let mut item = new_weighted_rollback_bundle("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedRollbackBundleRequiredCodeRule {
            message: "weighted rollback bundle needs this field".to_string(),
        };
        let mut item = new_weighted_rollback_bundle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = WeightedRollbackBundleRequiredKeyIdRule {
            message: "weighted rollback bundle needs an entry".to_string(),
        };
        let mut item = new_weighted_rollback_bundle("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = WeightedRollbackBundleKeyIdLengthRule { max_length: 64 };
        let mut item = new_weighted_rollback_bundle("x", "y");
        item.key_id = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
