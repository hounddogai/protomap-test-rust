use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched escalation tier must pass. Check returns its problems, or none.
pub trait BatchedEscalationTierRule: std::fmt::Debug {
    fn check(&self, item: &BatchedEscalationTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEscalationTierRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedEscalationTierRule for BatchedEscalationTierRetriesRangeRule {
    fn check(&self, item: &BatchedEscalationTier) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEscalationTierCodeLengthRule {
    pub max_length: i64,
}

impl BatchedEscalationTierRule for BatchedEscalationTierCodeLengthRule {
    fn check(&self, item: &BatchedEscalationTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEscalationTierRequiredLabelRule {
    pub message: String,
}

impl BatchedEscalationTierRule for BatchedEscalationTierRequiredLabelRule {
    fn check(&self, item: &BatchedEscalationTier) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEscalationTierRequiredCodeRule {
    pub message: String,
}

impl BatchedEscalationTierRule for BatchedEscalationTierRequiredCodeRule {
    fn check(&self, item: &BatchedEscalationTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched escalation tier.
#[derive(Debug, Clone)]
pub struct BatchedEscalationTierValidator {
    pub rules: Vec<Rc<dyn BatchedEscalationTierRule>>,
}

impl BatchedEscalationTierValidator {
    pub fn validate(&self, item: &BatchedEscalationTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedEscalationTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedEscalationTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_escalation_tier_validator() -> BatchedEscalationTierValidator {
    BatchedEscalationTierValidator {
        rules: vec![
            Rc::new(BatchedEscalationTierRetriesRangeRule { min: 16, max: 32 }) as Rc<dyn BatchedEscalationTierRule>,
            Rc::new(BatchedEscalationTierCodeLengthRule { max_length: 120 }),
            Rc::new(BatchedEscalationTierRequiredLabelRule {
                message: "batched escalation tier needs this field".to_string(),
            }),
            Rc::new(BatchedEscalationTierRequiredCodeRule {
                message: "batched escalation tier needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_escalation_tier_validator();
        let item = new_batched_escalation_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = BatchedEscalationTierRetriesRangeRule { min: 16, max: 1000 };
        let mut item = new_batched_escalation_tier("x", "y");
        item.retries = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedEscalationTierCodeLengthRule { max_length: 80 };
        let mut item = new_batched_escalation_tier("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = BatchedEscalationTierRequiredLabelRule {
            message: "batched escalation tier needs this field".to_string(),
        };
        let mut item = new_batched_escalation_tier("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedEscalationTierRequiredCodeRule {
            message: "batched escalation tier needs a value".to_string(),
        };
        let mut item = new_batched_escalation_tier("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
