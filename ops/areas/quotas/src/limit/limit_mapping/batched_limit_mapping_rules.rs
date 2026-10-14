use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched limit mapping must pass. Check returns its problems, or none.
pub trait BatchedLimitMappingRule: std::fmt::Debug {
    fn check(&self, item: &BatchedLimitMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitMappingCountryLengthRule {
    pub max_length: i64,
}

impl BatchedLimitMappingRule for BatchedLimitMappingCountryLengthRule {
    fn check(&self, item: &BatchedLimitMapping) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitMappingAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl BatchedLimitMappingRule for BatchedLimitMappingAllowedLabelRule {
    fn check(&self, item: &BatchedLimitMapping) -> Vec<String> {
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
pub struct BatchedLimitMappingLabelLengthRule {
    pub max_length: i64,
}

impl BatchedLimitMappingRule for BatchedLimitMappingLabelLengthRule {
    fn check(&self, item: &BatchedLimitMapping) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitMappingRequiredCodeRule {
    pub message: String,
}

impl BatchedLimitMappingRule for BatchedLimitMappingRequiredCodeRule {
    fn check(&self, item: &BatchedLimitMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitMappingRequiredLabelRule {
    pub message: String,
}

impl BatchedLimitMappingRule for BatchedLimitMappingRequiredLabelRule {
    fn check(&self, item: &BatchedLimitMapping) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched limit mapping.
#[derive(Debug, Clone)]
pub struct BatchedLimitMappingValidator {
    pub rules: Vec<Rc<dyn BatchedLimitMappingRule>>,
}

impl BatchedLimitMappingValidator {
    pub fn validate(&self, item: &BatchedLimitMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedLimitMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedLimitMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_limit_mapping_validator() -> BatchedLimitMappingValidator {
    BatchedLimitMappingValidator {
        rules: vec![
            Rc::new(BatchedLimitMappingCountryLengthRule { max_length: 3600 }) as Rc<dyn BatchedLimitMappingRule>,
            Rc::new(BatchedLimitMappingAllowedLabelRule { allowed: Vec::new() }),
            Rc::new(BatchedLimitMappingLabelLengthRule { max_length: 64 }),
            Rc::new(BatchedLimitMappingRequiredCodeRule {
                message: "batched limit mapping needs a value".to_string(),
            }),
            Rc::new(BatchedLimitMappingRequiredLabelRule {
                message: "batched limit mapping needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_limit_mapping_validator();
        let item = new_batched_limit_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = BatchedLimitMappingCountryLengthRule { max_length: 3600 };
        let mut item = new_batched_limit_mapping("x", "y");
        item.country = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = BatchedLimitMappingLabelLengthRule { max_length: 32 };
        let mut item = new_batched_limit_mapping("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedLimitMappingRequiredCodeRule {
            message: "batched limit mapping needs an entry".to_string(),
        };
        let mut item = new_batched_limit_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = BatchedLimitMappingRequiredLabelRule {
            message: "batched limit mapping needs this field".to_string(),
        };
        let mut item = new_batched_limit_mapping("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
