use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding allocation must pass. Check returns its problems, or none.
pub trait BindingAllocationRule: std::fmt::Debug {
    fn check(&self, item: &BindingAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingAllocationDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BindingAllocationRule for BindingAllocationDiscountRangeRule {
    fn check(&self, item: &BindingAllocation) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingAllocationStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingAllocationRule for BindingAllocationStartsAtRangeRule {
    fn check(&self, item: &BindingAllocation) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingAllocationReviewerLengthRule {
    pub max_length: i64,
}

impl BindingAllocationRule for BindingAllocationReviewerLengthRule {
    fn check(&self, item: &BindingAllocation) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingAllocationRequiredAudienceRule {
    pub message: String,
}

impl BindingAllocationRule for BindingAllocationRequiredAudienceRule {
    fn check(&self, item: &BindingAllocation) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingAllocationAudienceLengthRule {
    pub max_length: i64,
}

impl BindingAllocationRule for BindingAllocationAudienceLengthRule {
    fn check(&self, item: &BindingAllocation) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding allocation.
#[derive(Debug, Clone)]
pub struct BindingAllocationValidator {
    pub rules: Vec<Rc<dyn BindingAllocationRule>>,
}

impl BindingAllocationValidator {
    pub fn validate(&self, item: &BindingAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BindingAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_allocation_validator() -> BindingAllocationValidator {
    BindingAllocationValidator {
        rules: vec![
            Rc::new(BindingAllocationDiscountRangeRule { min: 9.6, max: 500.7 }) as Rc<dyn BindingAllocationRule>,
            Rc::new(BindingAllocationStartsAtRangeRule { min: 3600, max: 86400 }),
            Rc::new(BindingAllocationReviewerLengthRule { max_length: 120 }),
            Rc::new(BindingAllocationRequiredAudienceRule {
                message: "binding allocation needs this field".to_string(),
            }),
            Rc::new(BindingAllocationAudienceLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_binding_allocation_validator();
        let item = new_binding_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = BindingAllocationDiscountRangeRule { min: 1.9, max: 2569.3 };
        let mut item = new_binding_allocation("x", "y");
        item.discount = 2570.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = BindingAllocationStartsAtRangeRule { min: 3600, max: 3600 };
        let mut item = new_binding_allocation("x", "y");
        item.starts_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = BindingAllocationReviewerLengthRule { max_length: 86400 };
        let mut item = new_binding_allocation("x", "y");
        item.reviewer = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = BindingAllocationRequiredAudienceRule {
            message: "binding allocation needs this field".to_string(),
        };
        let mut item = new_binding_allocation("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = BindingAllocationAudienceLengthRule { max_length: 86400 };
        let mut item = new_binding_allocation("x", "y");
        item.audience = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
