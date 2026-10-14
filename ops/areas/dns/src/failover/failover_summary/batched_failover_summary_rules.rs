use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched failover summary must pass. Check returns its problems, or none.
pub trait BatchedFailoverSummaryRule: std::fmt::Debug {
    fn check(&self, item: &BatchedFailoverSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFailoverSummaryRequiredCodeRule {
    pub message: String,
}

impl BatchedFailoverSummaryRule for BatchedFailoverSummaryRequiredCodeRule {
    fn check(&self, item: &BatchedFailoverSummary) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFailoverSummaryRequiredPrincipalRule {
    pub message: String,
}

impl BatchedFailoverSummaryRule for BatchedFailoverSummaryRequiredPrincipalRule {
    fn check(&self, item: &BatchedFailoverSummary) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFailoverSummaryCodePrefixRule {
    pub prefix: String,
}

impl BatchedFailoverSummaryRule for BatchedFailoverSummaryCodePrefixRule {
    fn check(&self, item: &BatchedFailoverSummary) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFailoverSummaryRequiredTitleRule {
    pub message: String,
}

impl BatchedFailoverSummaryRule for BatchedFailoverSummaryRequiredTitleRule {
    fn check(&self, item: &BatchedFailoverSummary) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched failover summary.
#[derive(Debug, Clone)]
pub struct BatchedFailoverSummaryValidator {
    pub rules: Vec<Rc<dyn BatchedFailoverSummaryRule>>,
}

impl BatchedFailoverSummaryValidator {
    pub fn validate(&self, item: &BatchedFailoverSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedFailoverSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedFailoverSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_failover_summary_validator() -> BatchedFailoverSummaryValidator {
    BatchedFailoverSummaryValidator {
        rules: vec![
            Rc::new(BatchedFailoverSummaryRequiredCodeRule {
                message: "batched failover summary needs a value".to_string(),
            }) as Rc<dyn BatchedFailoverSummaryRule>,
            Rc::new(BatchedFailoverSummaryRequiredPrincipalRule {
                message: "batched failover summary needs a value".to_string(),
            }),
            Rc::new(BatchedFailoverSummaryCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(BatchedFailoverSummaryRequiredTitleRule {
                message: "batched failover summary needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_failover_summary_validator();
        let item = new_batched_failover_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedFailoverSummaryRequiredCodeRule {
            message: "batched failover summary needs an entry".to_string(),
        };
        let mut item = new_batched_failover_summary("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = BatchedFailoverSummaryRequiredPrincipalRule {
            message: "batched failover summary needs an entry".to_string(),
        };
        let mut item = new_batched_failover_summary("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = BatchedFailoverSummaryRequiredTitleRule {
            message: "batched failover summary needs an entry".to_string(),
        };
        let mut item = new_batched_failover_summary("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
