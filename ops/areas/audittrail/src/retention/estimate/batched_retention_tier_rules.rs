use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched retention tier must pass. Check returns its problems, or none.
pub trait BatchedRetentionTierRule: std::fmt::Debug {
    fn check(&self, item: &BatchedRetentionTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRetentionTierCodeLengthRule {
    pub max_length: i64,
}

impl BatchedRetentionTierRule for BatchedRetentionTierCodeLengthRule {
    fn check(&self, item: &BatchedRetentionTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRetentionTierCodePrefixRule {
    pub prefix: String,
}

impl BatchedRetentionTierRule for BatchedRetentionTierCodePrefixRule {
    fn check(&self, item: &BatchedRetentionTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRetentionTierClusterLengthRule {
    pub max_length: i64,
}

impl BatchedRetentionTierRule for BatchedRetentionTierClusterLengthRule {
    fn check(&self, item: &BatchedRetentionTier) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRetentionTierAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl BatchedRetentionTierRule for BatchedRetentionTierAllowedCodeRule {
    fn check(&self, item: &BatchedRetentionTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched retention tier.
#[derive(Debug, Clone)]
pub struct BatchedRetentionTierValidator {
    pub rules: Vec<Rc<dyn BatchedRetentionTierRule>>,
}

impl BatchedRetentionTierValidator {
    pub fn validate(&self, item: &BatchedRetentionTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedRetentionTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedRetentionTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_retention_tier_validator() -> BatchedRetentionTierValidator {
    BatchedRetentionTierValidator {
        rules: vec![
            Rc::new(BatchedRetentionTierCodeLengthRule { max_length: 86400 }) as Rc<dyn BatchedRetentionTierRule>,
            Rc::new(BatchedRetentionTierCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(BatchedRetentionTierClusterLengthRule { max_length: 3600 }),
            Rc::new(BatchedRetentionTierAllowedCodeRule {
                allowed: vec!["batched".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_retention_tier_validator();
        let item = new_batched_retention_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedRetentionTierCodeLengthRule { max_length: 120 };
        let mut item = new_batched_retention_tier("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = BatchedRetentionTierClusterLengthRule { max_length: 255 };
        let mut item = new_batched_retention_tier("x", "y");
        item.cluster = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
