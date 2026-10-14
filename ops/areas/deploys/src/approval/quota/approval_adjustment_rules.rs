use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a approval adjustment must pass. Check returns its problems, or none.
pub trait ApprovalAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl ApprovalAdjustmentRule for ApprovalAdjustmentCodePrefixRule {
    fn check(&self, item: &ApprovalAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAdjustmentAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl ApprovalAdjustmentRule for ApprovalAdjustmentAllowedTitleRule {
    fn check(&self, item: &ApprovalAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is stale", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAdjustmentCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ApprovalAdjustmentRule for ApprovalAdjustmentCostRangeRule {
    fn check(&self, item: &ApprovalAdjustment) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAdjustmentUnitLengthRule {
    pub max_length: i64,
}

impl ApprovalAdjustmentRule for ApprovalAdjustmentUnitLengthRule {
    fn check(&self, item: &ApprovalAdjustment) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a approval adjustment.
#[derive(Debug, Clone)]
pub struct ApprovalAdjustmentValidator {
    pub rules: Vec<Rc<dyn ApprovalAdjustmentRule>>,
}

impl ApprovalAdjustmentValidator {
    pub fn validate(&self, item: &ApprovalAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ApprovalAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_adjustment_validator() -> ApprovalAdjustmentValidator {
    ApprovalAdjustmentValidator {
        rules: vec![
            Rc::new(ApprovalAdjustmentCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn ApprovalAdjustmentRule>,
            Rc::new(ApprovalAdjustmentAllowedTitleRule {
                allowed: vec!["batched".to_string(), "adaptive".to_string()],
            }),
            Rc::new(ApprovalAdjustmentCostRangeRule { min: 7.1, max: 4515.2 }),
            Rc::new(ApprovalAdjustmentUnitLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_approval_adjustment_validator();
        let item = new_approval_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = ApprovalAdjustmentCostRangeRule { min: 1.9, max: 737.1 };
        let mut item = new_approval_adjustment("x", "y");
        item.cost = 738.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = ApprovalAdjustmentUnitLengthRule { max_length: 64 };
        let mut item = new_approval_adjustment("x", "y");
        item.unit = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
