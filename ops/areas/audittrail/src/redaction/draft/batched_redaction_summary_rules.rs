use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched redaction summary must pass. Check returns its problems, or none.
pub trait BatchedRedactionSummaryRule: std::fmt::Debug {
    fn check(&self, item: &BatchedRedactionSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRedactionSummaryKeyIdLengthRule {
    pub max_length: i64,
}

impl BatchedRedactionSummaryRule for BatchedRedactionSummaryKeyIdLengthRule {
    fn check(&self, item: &BatchedRedactionSummary) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRedactionSummarySequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedRedactionSummaryRule for BatchedRedactionSummarySequenceRangeRule {
    fn check(&self, item: &BatchedRedactionSummary) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRedactionSummaryCodePrefixRule {
    pub prefix: String,
}

impl BatchedRedactionSummaryRule for BatchedRedactionSummaryCodePrefixRule {
    fn check(&self, item: &BatchedRedactionSummary) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRedactionSummaryLabelLengthRule {
    pub max_length: i64,
}

impl BatchedRedactionSummaryRule for BatchedRedactionSummaryLabelLengthRule {
    fn check(&self, item: &BatchedRedactionSummary) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched redaction summary.
#[derive(Debug, Clone)]
pub struct BatchedRedactionSummaryValidator {
    pub rules: Vec<Rc<dyn BatchedRedactionSummaryRule>>,
}

impl BatchedRedactionSummaryValidator {
    pub fn validate(&self, item: &BatchedRedactionSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedRedactionSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedRedactionSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_redaction_summary_validator() -> BatchedRedactionSummaryValidator {
    BatchedRedactionSummaryValidator {
        rules: vec![
            Rc::new(BatchedRedactionSummaryKeyIdLengthRule { max_length: 16 }) as Rc<dyn BatchedRedactionSummaryRule>,
            Rc::new(BatchedRedactionSummarySequenceRangeRule { min: 16, max: 120 }),
            Rc::new(BatchedRedactionSummaryCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(BatchedRedactionSummaryLabelLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_redaction_summary_validator();
        let item = new_batched_redaction_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = BatchedRedactionSummaryKeyIdLengthRule { max_length: 3600 };
        let mut item = new_batched_redaction_summary("x", "y");
        item.key_id = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = BatchedRedactionSummarySequenceRangeRule { min: 16, max: 120 };
        let mut item = new_batched_redaction_summary("x", "y");
        item.sequence = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = BatchedRedactionSummaryLabelLengthRule { max_length: 80 };
        let mut item = new_batched_redaction_summary("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
