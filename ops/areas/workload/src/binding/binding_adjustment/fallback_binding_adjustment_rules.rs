use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback binding adjustment must pass. Check returns its problems, or none.
pub trait FallbackBindingAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &FallbackBindingAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingAdjustmentCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackBindingAdjustmentRule for FallbackBindingAdjustmentCapacityRangeRule {
    fn check(&self, item: &FallbackBindingAdjustment) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingAdjustmentAllowedNameRule {
    pub allowed: Vec<String>,
}

impl FallbackBindingAdjustmentRule for FallbackBindingAdjustmentAllowedNameRule {
    fn check(&self, item: &FallbackBindingAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is too long", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingAdjustmentRequiredCodeRule {
    pub message: String,
}

impl FallbackBindingAdjustmentRule for FallbackBindingAdjustmentRequiredCodeRule {
    fn check(&self, item: &FallbackBindingAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl FallbackBindingAdjustmentRule for FallbackBindingAdjustmentCodeLengthRule {
    fn check(&self, item: &FallbackBindingAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback binding adjustment.
#[derive(Debug, Clone)]
pub struct FallbackBindingAdjustmentValidator {
    pub rules: Vec<Rc<dyn FallbackBindingAdjustmentRule>>,
}

impl FallbackBindingAdjustmentValidator {
    pub fn validate(&self, item: &FallbackBindingAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackBindingAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackBindingAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_binding_adjustment_validator() -> FallbackBindingAdjustmentValidator {
    FallbackBindingAdjustmentValidator {
        rules: vec![
            Rc::new(FallbackBindingAdjustmentCapacityRangeRule { min: 80, max: 1000 })
                as Rc<dyn FallbackBindingAdjustmentRule>,
            Rc::new(FallbackBindingAdjustmentAllowedNameRule {
                allowed: vec!["global".to_string()],
            }),
            Rc::new(FallbackBindingAdjustmentRequiredCodeRule {
                message: "fallback binding adjustment needs this field".to_string(),
            }),
            Rc::new(FallbackBindingAdjustmentCodeLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_binding_adjustment_validator();
        let item = new_fallback_binding_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = FallbackBindingAdjustmentCapacityRangeRule { min: 64, max: 86400 };
        let mut item = new_fallback_binding_adjustment("x", "y");
        item.capacity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackBindingAdjustmentRequiredCodeRule {
            message: "fallback binding adjustment needs this field".to_string(),
        };
        let mut item = new_fallback_binding_adjustment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackBindingAdjustmentCodeLengthRule { max_length: 80 };
        let mut item = new_fallback_binding_adjustment("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
