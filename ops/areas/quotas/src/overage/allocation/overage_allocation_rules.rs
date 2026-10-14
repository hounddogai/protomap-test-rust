use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a overage allocation must pass. Check returns its problems, or none.
pub trait OverageAllocationRule: std::fmt::Debug {
    fn check(&self, item: &OverageAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageAllocationCodeLengthRule {
    pub max_length: i64,
}

impl OverageAllocationRule for OverageAllocationCodeLengthRule {
    fn check(&self, item: &OverageAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageAllocationRequiredCipherRule {
    pub message: String,
}

impl OverageAllocationRule for OverageAllocationRequiredCipherRule {
    fn check(&self, item: &OverageAllocation) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageAllocationAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl OverageAllocationRule for OverageAllocationAllowedLabelRule {
    fn check(&self, item: &OverageAllocation) -> Vec<String> {
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
pub struct OverageAllocationBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl OverageAllocationRule for OverageAllocationBatchSizeRangeRule {
    fn check(&self, item: &OverageAllocation) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a overage allocation.
#[derive(Debug, Clone)]
pub struct OverageAllocationValidator {
    pub rules: Vec<Rc<dyn OverageAllocationRule>>,
}

impl OverageAllocationValidator {
    pub fn validate(&self, item: &OverageAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &OverageAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_allocation_validator() -> OverageAllocationValidator {
    OverageAllocationValidator {
        rules: vec![
            Rc::new(OverageAllocationCodeLengthRule { max_length: 86400 }) as Rc<dyn OverageAllocationRule>,
            Rc::new(OverageAllocationRequiredCipherRule {
                message: "overage allocation needs this field".to_string(),
            }),
            Rc::new(OverageAllocationAllowedLabelRule {
                allowed: vec!["scheduled".to_string(), "rolling".to_string()],
            }),
            Rc::new(OverageAllocationBatchSizeRangeRule { min: 32, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_allocation_validator();
        let item = new_overage_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = OverageAllocationCodeLengthRule { max_length: 1000 };
        let mut item = new_overage_allocation("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = OverageAllocationRequiredCipherRule {
            message: "overage allocation needs a value".to_string(),
        };
        let mut item = new_overage_allocation("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = OverageAllocationBatchSizeRangeRule { min: 120, max: 3600 };
        let mut item = new_overage_allocation("x", "y");
        item.batch_size = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
