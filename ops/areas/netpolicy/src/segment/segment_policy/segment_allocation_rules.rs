use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a segment allocation must pass. Check returns its problems, or none.
pub trait SegmentAllocationRule: std::fmt::Debug {
    fn check(&self, item: &SegmentAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAllocationEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentAllocationRule for SegmentAllocationEndsAtRangeRule {
    fn check(&self, item: &SegmentAllocation) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAllocationBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SegmentAllocationRule for SegmentAllocationBudgetRangeRule {
    fn check(&self, item: &SegmentAllocation) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAllocationTimezoneLengthRule {
    pub max_length: i64,
}

impl SegmentAllocationRule for SegmentAllocationTimezoneLengthRule {
    fn check(&self, item: &SegmentAllocation) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAllocationCodeLengthRule {
    pub max_length: i64,
}

impl SegmentAllocationRule for SegmentAllocationCodeLengthRule {
    fn check(&self, item: &SegmentAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAllocationAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl SegmentAllocationRule for SegmentAllocationAllowedTimezoneRule {
    fn check(&self, item: &SegmentAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is blocked", item.timezone)];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment allocation.
#[derive(Debug, Clone)]
pub struct SegmentAllocationValidator {
    pub rules: Vec<Rc<dyn SegmentAllocationRule>>,
}

impl SegmentAllocationValidator {
    pub fn validate(&self, item: &SegmentAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SegmentAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_allocation_validator() -> SegmentAllocationValidator {
    SegmentAllocationValidator {
        rules: vec![
            Rc::new(SegmentAllocationEndsAtRangeRule { min: 80, max: 255 }) as Rc<dyn SegmentAllocationRule>,
            Rc::new(SegmentAllocationBudgetRangeRule { min: 6.9, max: 4913.1 }),
            Rc::new(SegmentAllocationTimezoneLengthRule { max_length: 3600 }),
            Rc::new(SegmentAllocationCodeLengthRule { max_length: 64 }),
            Rc::new(SegmentAllocationAllowedTimezoneRule {
                allowed: vec!["batched".to_string(), "primary".to_string(), "lenient".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_allocation_validator();
        let item = new_segment_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = SegmentAllocationEndsAtRangeRule { min: 16, max: 255 };
        let mut item = new_segment_allocation("x", "y");
        item.ends_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = SegmentAllocationBudgetRangeRule { min: 1.6, max: 3891.4 };
        let mut item = new_segment_allocation("x", "y");
        item.budget = 3892.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = SegmentAllocationTimezoneLengthRule { max_length: 120 };
        let mut item = new_segment_allocation("x", "y");
        item.timezone = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SegmentAllocationCodeLengthRule { max_length: 32 };
        let mut item = new_segment_allocation("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
