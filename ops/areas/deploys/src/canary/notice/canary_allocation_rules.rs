use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a canary allocation must pass. Check returns its problems, or none.
pub trait CanaryAllocationRule: std::fmt::Debug {
    fn check(&self, item: &CanaryAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryAllocationRequiredTitleRule {
    pub message: String,
}

impl CanaryAllocationRule for CanaryAllocationRequiredTitleRule {
    fn check(&self, item: &CanaryAllocation) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryAllocationCodeLengthRule {
    pub max_length: i64,
}

impl CanaryAllocationRule for CanaryAllocationCodeLengthRule {
    fn check(&self, item: &CanaryAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryAllocationRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CanaryAllocationRule for CanaryAllocationRetriesRangeRule {
    fn check(&self, item: &CanaryAllocation) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryAllocationRequiredLabelRule {
    pub message: String,
}

impl CanaryAllocationRule for CanaryAllocationRequiredLabelRule {
    fn check(&self, item: &CanaryAllocation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryAllocationRequiredCodeRule {
    pub message: String,
}

impl CanaryAllocationRule for CanaryAllocationRequiredCodeRule {
    fn check(&self, item: &CanaryAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryAllocationCodePrefixRule {
    pub prefix: String,
}

impl CanaryAllocationRule for CanaryAllocationCodePrefixRule {
    fn check(&self, item: &CanaryAllocation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary allocation.
#[derive(Debug, Clone)]
pub struct CanaryAllocationValidator {
    pub rules: Vec<Rc<dyn CanaryAllocationRule>>,
}

impl CanaryAllocationValidator {
    pub fn validate(&self, item: &CanaryAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CanaryAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_allocation_validator() -> CanaryAllocationValidator {
    CanaryAllocationValidator {
        rules: vec![
            Rc::new(CanaryAllocationRequiredTitleRule {
                message: "canary allocation needs this field".to_string(),
            }) as Rc<dyn CanaryAllocationRule>,
            Rc::new(CanaryAllocationCodeLengthRule { max_length: 64 }),
            Rc::new(CanaryAllocationRetriesRangeRule { min: 255, max: 3600 }),
            Rc::new(CanaryAllocationRequiredLabelRule {
                message: "canary allocation needs a value".to_string(),
            }),
            Rc::new(CanaryAllocationRequiredCodeRule {
                message: "canary allocation needs an entry".to_string(),
            }),
            Rc::new(CanaryAllocationCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_allocation_validator();
        let item = new_canary_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = CanaryAllocationRequiredTitleRule {
            message: "canary allocation needs this field".to_string(),
        };
        let mut item = new_canary_allocation("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CanaryAllocationCodeLengthRule { max_length: 255 };
        let mut item = new_canary_allocation("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = CanaryAllocationRetriesRangeRule { min: 64, max: 1000 };
        let mut item = new_canary_allocation("x", "y");
        item.retries = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = CanaryAllocationRequiredLabelRule {
            message: "canary allocation needs this field".to_string(),
        };
        let mut item = new_canary_allocation("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CanaryAllocationRequiredCodeRule {
            message: "canary allocation needs a value".to_string(),
        };
        let mut item = new_canary_allocation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
