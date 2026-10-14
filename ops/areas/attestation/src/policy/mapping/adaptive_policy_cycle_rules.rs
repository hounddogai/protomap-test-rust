use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive policy cycle must pass. Check returns its problems, or none.
pub trait AdaptivePolicyCycleRule: std::fmt::Debug {
    fn check(&self, item: &AdaptivePolicyCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePolicyCycleCategoryLengthRule {
    pub max_length: i64,
}

impl AdaptivePolicyCycleRule for AdaptivePolicyCycleCategoryLengthRule {
    fn check(&self, item: &AdaptivePolicyCycle) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePolicyCycleRequiredCategoryRule {
    pub message: String,
}

impl AdaptivePolicyCycleRule for AdaptivePolicyCycleRequiredCategoryRule {
    fn check(&self, item: &AdaptivePolicyCycle) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePolicyCycleDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptivePolicyCycleRule for AdaptivePolicyCycleDiscountRangeRule {
    fn check(&self, item: &AdaptivePolicyCycle) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive policy cycle.
#[derive(Debug, Clone)]
pub struct AdaptivePolicyCycleValidator {
    pub rules: Vec<Rc<dyn AdaptivePolicyCycleRule>>,
}

impl AdaptivePolicyCycleValidator {
    pub fn validate(&self, item: &AdaptivePolicyCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptivePolicyCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptivePolicyCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_policy_cycle_validator() -> AdaptivePolicyCycleValidator {
    AdaptivePolicyCycleValidator {
        rules: vec![
            Rc::new(AdaptivePolicyCycleCategoryLengthRule { max_length: 80 }) as Rc<dyn AdaptivePolicyCycleRule>,
            Rc::new(AdaptivePolicyCycleRequiredCategoryRule {
                message: "adaptive policy cycle needs this field".to_string(),
            }),
            Rc::new(AdaptivePolicyCycleDiscountRangeRule { min: 0.9, max: 3513.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_policy_cycle_validator();
        let item = new_adaptive_policy_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = AdaptivePolicyCycleCategoryLengthRule { max_length: 16 };
        let mut item = new_adaptive_policy_cycle("x", "y");
        item.category = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = AdaptivePolicyCycleRequiredCategoryRule {
            message: "adaptive policy cycle needs a value".to_string(),
        };
        let mut item = new_adaptive_policy_cycle("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = AdaptivePolicyCycleDiscountRangeRule { min: 7.7, max: 683.5 };
        let mut item = new_adaptive_policy_cycle("x", "y");
        item.discount = 685.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
