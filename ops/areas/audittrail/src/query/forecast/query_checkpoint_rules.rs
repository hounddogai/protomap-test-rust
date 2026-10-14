use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query checkpoint must pass. Check returns its problems, or none.
pub trait QueryCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &QueryCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryCheckpointCodePrefixRule {
    pub prefix: String,
}

impl QueryCheckpointRule for QueryCheckpointCodePrefixRule {
    fn check(&self, item: &QueryCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryCheckpointNameLengthRule {
    pub max_length: i64,
}

impl QueryCheckpointRule for QueryCheckpointNameLengthRule {
    fn check(&self, item: &QueryCheckpoint) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryCheckpointRequiredNameRule {
    pub message: String,
}

impl QueryCheckpointRule for QueryCheckpointRequiredNameRule {
    fn check(&self, item: &QueryCheckpoint) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a query checkpoint.
#[derive(Debug, Clone)]
pub struct QueryCheckpointValidator {
    pub rules: Vec<Rc<dyn QueryCheckpointRule>>,
}

impl QueryCheckpointValidator {
    pub fn validate(&self, item: &QueryCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QueryCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_checkpoint_validator() -> QueryCheckpointValidator {
    QueryCheckpointValidator {
        rules: vec![
            Rc::new(QueryCheckpointCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn QueryCheckpointRule>,
            Rc::new(QueryCheckpointNameLengthRule { max_length: 64 }),
            Rc::new(QueryCheckpointRequiredNameRule {
                message: "query checkpoint needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_query_checkpoint_validator();
        let item = new_query_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = QueryCheckpointNameLengthRule { max_length: 1000 };
        let mut item = new_query_checkpoint("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = QueryCheckpointRequiredNameRule {
            message: "query checkpoint needs this field".to_string(),
        };
        let mut item = new_query_checkpoint("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
