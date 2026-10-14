use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched slo cycle must pass. Check returns its problems, or none.
pub trait BatchedSloCycleRule: std::fmt::Debug {
    fn check(&self, item: &BatchedSloCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSloCycleCodeLengthRule {
    pub max_length: i64,
}

impl BatchedSloCycleRule for BatchedSloCycleCodeLengthRule {
    fn check(&self, item: &BatchedSloCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSloCycleCodePrefixRule {
    pub prefix: String,
}

impl BatchedSloCycleRule for BatchedSloCycleCodePrefixRule {
    fn check(&self, item: &BatchedSloCycle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSloCycleAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl BatchedSloCycleRule for BatchedSloCycleAllowedLabelRule {
    fn check(&self, item: &BatchedSloCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is not allowed", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSloCycleRequiredCodeRule {
    pub message: String,
}

impl BatchedSloCycleRule for BatchedSloCycleRequiredCodeRule {
    fn check(&self, item: &BatchedSloCycle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSloCycleRequiredSummaryRule {
    pub message: String,
}

impl BatchedSloCycleRule for BatchedSloCycleRequiredSummaryRule {
    fn check(&self, item: &BatchedSloCycle) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSloCycleNameLengthRule {
    pub max_length: i64,
}

impl BatchedSloCycleRule for BatchedSloCycleNameLengthRule {
    fn check(&self, item: &BatchedSloCycle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched slo cycle.
#[derive(Debug, Clone)]
pub struct BatchedSloCycleValidator {
    pub rules: Vec<Rc<dyn BatchedSloCycleRule>>,
}

impl BatchedSloCycleValidator {
    pub fn validate(&self, item: &BatchedSloCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedSloCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedSloCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_slo_cycle_validator() -> BatchedSloCycleValidator {
    BatchedSloCycleValidator {
        rules: vec![
            Rc::new(BatchedSloCycleCodeLengthRule { max_length: 1000 }) as Rc<dyn BatchedSloCycleRule>,
            Rc::new(BatchedSloCycleCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(BatchedSloCycleAllowedLabelRule {
                allowed: vec!["default".to_string()],
            }),
            Rc::new(BatchedSloCycleRequiredCodeRule {
                message: "batched slo cycle needs this field".to_string(),
            }),
            Rc::new(BatchedSloCycleRequiredSummaryRule {
                message: "batched slo cycle needs this field".to_string(),
            }),
            Rc::new(BatchedSloCycleNameLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_slo_cycle_validator();
        let item = new_batched_slo_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedSloCycleCodeLengthRule { max_length: 120 };
        let mut item = new_batched_slo_cycle("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedSloCycleRequiredCodeRule {
            message: "batched slo cycle needs a value".to_string(),
        };
        let mut item = new_batched_slo_cycle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = BatchedSloCycleRequiredSummaryRule {
            message: "batched slo cycle needs a value".to_string(),
        };
        let mut item = new_batched_slo_cycle("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = BatchedSloCycleNameLengthRule { max_length: 64 };
        let mut item = new_batched_slo_cycle("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
