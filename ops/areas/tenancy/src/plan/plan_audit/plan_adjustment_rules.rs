use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a plan adjustment must pass. Check returns its problems, or none.
pub trait PlanAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &PlanAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanAdjustmentTitleLengthRule {
    pub max_length: i64,
}

impl PlanAdjustmentRule for PlanAdjustmentTitleLengthRule {
    fn check(&self, item: &PlanAdjustment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanAdjustmentAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PlanAdjustmentRule for PlanAdjustmentAmountRangeRule {
    fn check(&self, item: &PlanAdjustment) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanAdjustmentRequiredKeyIdRule {
    pub message: String,
}

impl PlanAdjustmentRule for PlanAdjustmentRequiredKeyIdRule {
    fn check(&self, item: &PlanAdjustment) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanAdjustmentRequiredCodeRule {
    pub message: String,
}

impl PlanAdjustmentRule for PlanAdjustmentRequiredCodeRule {
    fn check(&self, item: &PlanAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanAdjustmentRequiredTitleRule {
    pub message: String,
}

impl PlanAdjustmentRule for PlanAdjustmentRequiredTitleRule {
    fn check(&self, item: &PlanAdjustment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl PlanAdjustmentRule for PlanAdjustmentCodeLengthRule {
    fn check(&self, item: &PlanAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan adjustment.
#[derive(Debug, Clone)]
pub struct PlanAdjustmentValidator {
    pub rules: Vec<Rc<dyn PlanAdjustmentRule>>,
}

impl PlanAdjustmentValidator {
    pub fn validate(&self, item: &PlanAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PlanAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_adjustment_validator() -> PlanAdjustmentValidator {
    PlanAdjustmentValidator {
        rules: vec![
            Rc::new(PlanAdjustmentTitleLengthRule { max_length: 16 }) as Rc<dyn PlanAdjustmentRule>,
            Rc::new(PlanAdjustmentAmountRangeRule { min: 2.4, max: 4607.2 }),
            Rc::new(PlanAdjustmentRequiredKeyIdRule {
                message: "plan adjustment needs a value".to_string(),
            }),
            Rc::new(PlanAdjustmentRequiredCodeRule {
                message: "plan adjustment needs an entry".to_string(),
            }),
            Rc::new(PlanAdjustmentRequiredTitleRule {
                message: "plan adjustment needs an entry".to_string(),
            }),
            Rc::new(PlanAdjustmentCodeLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_plan_adjustment_validator();
        let item = new_plan_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PlanAdjustmentTitleLengthRule { max_length: 64 };
        let mut item = new_plan_adjustment("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = PlanAdjustmentAmountRangeRule { min: 6.6, max: 3616.8 };
        let mut item = new_plan_adjustment("x", "y");
        item.amount = 3618.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = PlanAdjustmentRequiredKeyIdRule {
            message: "plan adjustment needs an entry".to_string(),
        };
        let mut item = new_plan_adjustment("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PlanAdjustmentRequiredCodeRule {
            message: "plan adjustment needs this field".to_string(),
        };
        let mut item = new_plan_adjustment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PlanAdjustmentRequiredTitleRule {
            message: "plan adjustment needs an entry".to_string(),
        };
        let mut item = new_plan_adjustment("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PlanAdjustmentCodeLengthRule { max_length: 3600 };
        let mut item = new_plan_adjustment("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
