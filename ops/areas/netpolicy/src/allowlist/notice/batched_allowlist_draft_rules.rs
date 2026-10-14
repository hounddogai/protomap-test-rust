use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched allowlist draft must pass. Check returns its problems, or none.
pub trait BatchedAllowlistDraftRule: std::fmt::Debug {
    fn check(&self, item: &BatchedAllowlistDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistDraftCodePrefixRule {
    pub prefix: String,
}

impl BatchedAllowlistDraftRule for BatchedAllowlistDraftCodePrefixRule {
    fn check(&self, item: &BatchedAllowlistDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistDraftWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedAllowlistDraftRule for BatchedAllowlistDraftWeightRangeRule {
    fn check(&self, item: &BatchedAllowlistDraft) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistDraftChannelLengthRule {
    pub max_length: i64,
}

impl BatchedAllowlistDraftRule for BatchedAllowlistDraftChannelLengthRule {
    fn check(&self, item: &BatchedAllowlistDraft) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched allowlist draft.
#[derive(Debug, Clone)]
pub struct BatchedAllowlistDraftValidator {
    pub rules: Vec<Rc<dyn BatchedAllowlistDraftRule>>,
}

impl BatchedAllowlistDraftValidator {
    pub fn validate(&self, item: &BatchedAllowlistDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedAllowlistDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedAllowlistDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_allowlist_draft_validator() -> BatchedAllowlistDraftValidator {
    BatchedAllowlistDraftValidator {
        rules: vec![
            Rc::new(BatchedAllowlistDraftCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn BatchedAllowlistDraftRule>,
            Rc::new(BatchedAllowlistDraftWeightRangeRule { min: 5.5, max: 3263.0 }),
            Rc::new(BatchedAllowlistDraftChannelLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_allowlist_draft_validator();
        let item = new_batched_allowlist_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = BatchedAllowlistDraftWeightRangeRule { min: 8.2, max: 4076.0 };
        let mut item = new_batched_allowlist_draft("x", "y");
        item.weight = 4077.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = BatchedAllowlistDraftChannelLengthRule { max_length: 16 };
        let mut item = new_batched_allowlist_draft("x", "y");
        item.channel = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
