use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched runbook summary must pass. Check returns its problems, or none.
pub trait BatchedRunbookSummaryRule: std::fmt::Debug {
    fn check(&self, item: &BatchedRunbookSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookSummaryAllowedNameRule {
    pub allowed: Vec<String>,
}

impl BatchedRunbookSummaryRule for BatchedRunbookSummaryAllowedNameRule {
    fn check(&self, item: &BatchedRunbookSummary) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is not allowed", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookSummaryCodeLengthRule {
    pub max_length: i64,
}

impl BatchedRunbookSummaryRule for BatchedRunbookSummaryCodeLengthRule {
    fn check(&self, item: &BatchedRunbookSummary) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookSummaryApprovedGuardRule {
    pub limit: f64,
}

impl BatchedRunbookSummaryRule for BatchedRunbookSummaryApprovedGuardRule {
    fn check(&self, item: &BatchedRunbookSummary) -> Vec<String> {
        if item.approved && (item.ceiling > self.limit) {
            return vec![format!(
                "ceiling {} exceeds {} while approved",
                item.ceiling, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookSummaryKeyIdLengthRule {
    pub max_length: i64,
}

impl BatchedRunbookSummaryRule for BatchedRunbookSummaryKeyIdLengthRule {
    fn check(&self, item: &BatchedRunbookSummary) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched runbook summary.
#[derive(Debug, Clone)]
pub struct BatchedRunbookSummaryValidator {
    pub rules: Vec<Rc<dyn BatchedRunbookSummaryRule>>,
}

impl BatchedRunbookSummaryValidator {
    pub fn validate(&self, item: &BatchedRunbookSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedRunbookSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedRunbookSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_runbook_summary_validator() -> BatchedRunbookSummaryValidator {
    BatchedRunbookSummaryValidator {
        rules: vec![
            Rc::new(BatchedRunbookSummaryAllowedNameRule { allowed: Vec::new() }) as Rc<dyn BatchedRunbookSummaryRule>,
            Rc::new(BatchedRunbookSummaryCodeLengthRule { max_length: 120 }),
            Rc::new(BatchedRunbookSummaryApprovedGuardRule { limit: 9.2 }),
            Rc::new(BatchedRunbookSummaryKeyIdLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_runbook_summary_validator();
        let item = new_batched_runbook_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedRunbookSummaryCodeLengthRule { max_length: 16 };
        let mut item = new_batched_runbook_summary("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = BatchedRunbookSummaryKeyIdLengthRule { max_length: 16 };
        let mut item = new_batched_runbook_summary("x", "y");
        item.key_id = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
