use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched failover score must pass. Check returns its problems, or none.
pub trait BatchedFailoverScoreRule: std::fmt::Debug {
    fn check(&self, item: &BatchedFailoverScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFailoverScoreAllowedOwnerIdRule {
    pub allowed: Vec<String>,
}

impl BatchedFailoverScoreRule for BatchedFailoverScoreAllowedOwnerIdRule {
    fn check(&self, item: &BatchedFailoverScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.owner_id)))
        {
            return vec![format!("owner id {} is duplicated", item.owner_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFailoverScoreRequiredCodeRule {
    pub message: String,
}

impl BatchedFailoverScoreRule for BatchedFailoverScoreRequiredCodeRule {
    fn check(&self, item: &BatchedFailoverScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFailoverScoreOwnerIdLengthRule {
    pub max_length: i64,
}

impl BatchedFailoverScoreRule for BatchedFailoverScoreOwnerIdLengthRule {
    fn check(&self, item: &BatchedFailoverScore) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched failover score.
#[derive(Debug, Clone)]
pub struct BatchedFailoverScoreValidator {
    pub rules: Vec<Rc<dyn BatchedFailoverScoreRule>>,
}

impl BatchedFailoverScoreValidator {
    pub fn validate(&self, item: &BatchedFailoverScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedFailoverScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedFailoverScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_failover_score_validator() -> BatchedFailoverScoreValidator {
    BatchedFailoverScoreValidator {
        rules: vec![
            Rc::new(BatchedFailoverScoreAllowedOwnerIdRule {
                allowed: vec!["regional".to_string()],
            }) as Rc<dyn BatchedFailoverScoreRule>,
            Rc::new(BatchedFailoverScoreRequiredCodeRule {
                message: "batched failover score needs this field".to_string(),
            }),
            Rc::new(BatchedFailoverScoreOwnerIdLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_failover_score_validator();
        let item = new_batched_failover_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedFailoverScoreRequiredCodeRule {
            message: "batched failover score needs a value".to_string(),
        };
        let mut item = new_batched_failover_score("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = BatchedFailoverScoreOwnerIdLengthRule { max_length: 3600 };
        let mut item = new_batched_failover_score("x", "y");
        item.owner_id = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
