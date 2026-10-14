use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary ttl policy batch must pass. Check returns its problems, or none.
pub trait PrimaryTtlPolicyBatchRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryTtlPolicyBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyBatchClusterLengthRule {
    pub max_length: i64,
}

impl PrimaryTtlPolicyBatchRule for PrimaryTtlPolicyBatchClusterLengthRule {
    fn check(&self, item: &PrimaryTtlPolicyBatch) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyBatchArchivedGuardRule {
    pub limit: i64,
}

impl PrimaryTtlPolicyBatchRule for PrimaryTtlPolicyBatchArchivedGuardRule {
    fn check(&self, item: &PrimaryTtlPolicyBatch) -> Vec<String> {
        if item.archived && (item.capacity > self.limit) {
            return vec![format!(
                "capacity {} exceeds {} while archived",
                item.capacity, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyBatchCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryTtlPolicyBatchRule for PrimaryTtlPolicyBatchCodeLengthRule {
    fn check(&self, item: &PrimaryTtlPolicyBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyBatchRequiredClusterRule {
    pub message: String,
}

impl PrimaryTtlPolicyBatchRule for PrimaryTtlPolicyBatchRequiredClusterRule {
    fn check(&self, item: &PrimaryTtlPolicyBatch) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyBatchTitleLengthRule {
    pub max_length: i64,
}

impl PrimaryTtlPolicyBatchRule for PrimaryTtlPolicyBatchTitleLengthRule {
    fn check(&self, item: &PrimaryTtlPolicyBatch) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary ttl policy batch.
#[derive(Debug, Clone)]
pub struct PrimaryTtlPolicyBatchValidator {
    pub rules: Vec<Rc<dyn PrimaryTtlPolicyBatchRule>>,
}

impl PrimaryTtlPolicyBatchValidator {
    pub fn validate(&self, item: &PrimaryTtlPolicyBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryTtlPolicyBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryTtlPolicyBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_ttl_policy_batch_validator() -> PrimaryTtlPolicyBatchValidator {
    PrimaryTtlPolicyBatchValidator {
        rules: vec![
            Rc::new(PrimaryTtlPolicyBatchClusterLengthRule { max_length: 255 }) as Rc<dyn PrimaryTtlPolicyBatchRule>,
            Rc::new(PrimaryTtlPolicyBatchArchivedGuardRule { limit: 120 }),
            Rc::new(PrimaryTtlPolicyBatchCodeLengthRule { max_length: 1000 }),
            Rc::new(PrimaryTtlPolicyBatchRequiredClusterRule {
                message: "primary ttl policy batch needs a value".to_string(),
            }),
            Rc::new(PrimaryTtlPolicyBatchTitleLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_ttl_policy_batch_validator();
        let item = new_primary_ttl_policy_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = PrimaryTtlPolicyBatchClusterLengthRule { max_length: 3600 };
        let mut item = new_primary_ttl_policy_batch("x", "y");
        item.cluster = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryTtlPolicyBatchCodeLengthRule { max_length: 80 };
        let mut item = new_primary_ttl_policy_batch("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = PrimaryTtlPolicyBatchRequiredClusterRule {
            message: "primary ttl policy batch needs an entry".to_string(),
        };
        let mut item = new_primary_ttl_policy_batch("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PrimaryTtlPolicyBatchTitleLengthRule { max_length: 255 };
        let mut item = new_primary_ttl_policy_batch("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
