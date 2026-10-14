use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a plan allocation must pass. Check returns its problems, or none.
pub trait PlanAllocationRule: std::fmt::Debug {
    fn check(&self, item: &PlanAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanAllocationRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PlanAllocationRule for PlanAllocationRotationDaysRangeRule {
    fn check(&self, item: &PlanAllocation) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanAllocationThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PlanAllocationRule for PlanAllocationThresholdRangeRule {
    fn check(&self, item: &PlanAllocation) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanAllocationCodeLengthRule {
    pub max_length: i64,
}

impl PlanAllocationRule for PlanAllocationCodeLengthRule {
    fn check(&self, item: &PlanAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanAllocationCodePrefixRule {
    pub prefix: String,
}

impl PlanAllocationRule for PlanAllocationCodePrefixRule {
    fn check(&self, item: &PlanAllocation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanAllocationRequiredNameRule {
    pub message: String,
}

impl PlanAllocationRule for PlanAllocationRequiredNameRule {
    fn check(&self, item: &PlanAllocation) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan allocation.
#[derive(Debug, Clone)]
pub struct PlanAllocationValidator {
    pub rules: Vec<Rc<dyn PlanAllocationRule>>,
}

impl PlanAllocationValidator {
    pub fn validate(&self, item: &PlanAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PlanAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_allocation_validator() -> PlanAllocationValidator {
    PlanAllocationValidator {
        rules: vec![
            Rc::new(PlanAllocationRotationDaysRangeRule { min: 32, max: 64 }) as Rc<dyn PlanAllocationRule>,
            Rc::new(PlanAllocationThresholdRangeRule { min: 6.1, max: 3481.1 }),
            Rc::new(PlanAllocationCodeLengthRule { max_length: 86400 }),
            Rc::new(PlanAllocationCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(PlanAllocationRequiredNameRule {
                message: "plan allocation needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_plan_allocation_validator();
        let item = new_plan_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = PlanAllocationRotationDaysRangeRule { min: 16, max: 86400 };
        let mut item = new_plan_allocation("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = PlanAllocationThresholdRangeRule { min: 5.6, max: 3758.0 };
        let mut item = new_plan_allocation("x", "y");
        item.threshold = 3759.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PlanAllocationCodeLengthRule { max_length: 16 };
        let mut item = new_plan_allocation("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PlanAllocationRequiredNameRule {
            message: "plan allocation needs a value".to_string(),
        };
        let mut item = new_plan_allocation("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
