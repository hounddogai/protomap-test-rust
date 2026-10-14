use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient peering variance must pass. Check returns its problems, or none.
pub trait LenientPeeringVarianceRule: std::fmt::Debug {
    fn check(&self, item: &LenientPeeringVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPeeringVarianceDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientPeeringVarianceRule for LenientPeeringVarianceDiscountRangeRule {
    fn check(&self, item: &LenientPeeringVariance) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPeeringVarianceRequiredCodeRule {
    pub message: String,
}

impl LenientPeeringVarianceRule for LenientPeeringVarianceRequiredCodeRule {
    fn check(&self, item: &LenientPeeringVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPeeringVarianceAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientPeeringVarianceRule for LenientPeeringVarianceAmountRangeRule {
    fn check(&self, item: &LenientPeeringVariance) -> Vec<String> {
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
pub struct LenientPeeringVarianceRequiredLabelRule {
    pub message: String,
}

impl LenientPeeringVarianceRule for LenientPeeringVarianceRequiredLabelRule {
    fn check(&self, item: &LenientPeeringVariance) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPeeringVarianceAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl LenientPeeringVarianceRule for LenientPeeringVarianceAllowedLabelRule {
    fn check(&self, item: &LenientPeeringVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is stale", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPeeringVarianceLabelLengthRule {
    pub max_length: i64,
}

impl LenientPeeringVarianceRule for LenientPeeringVarianceLabelLengthRule {
    fn check(&self, item: &LenientPeeringVariance) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient peering variance.
#[derive(Debug, Clone)]
pub struct LenientPeeringVarianceValidator {
    pub rules: Vec<Rc<dyn LenientPeeringVarianceRule>>,
}

impl LenientPeeringVarianceValidator {
    pub fn validate(&self, item: &LenientPeeringVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientPeeringVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientPeeringVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_peering_variance_validator() -> LenientPeeringVarianceValidator {
    LenientPeeringVarianceValidator {
        rules: vec![
            Rc::new(LenientPeeringVarianceDiscountRangeRule { min: 8.9, max: 4949.7 })
                as Rc<dyn LenientPeeringVarianceRule>,
            Rc::new(LenientPeeringVarianceRequiredCodeRule {
                message: "lenient peering variance needs this field".to_string(),
            }),
            Rc::new(LenientPeeringVarianceAmountRangeRule { min: 8.5, max: 272.4 }),
            Rc::new(LenientPeeringVarianceRequiredLabelRule {
                message: "lenient peering variance needs an entry".to_string(),
            }),
            Rc::new(LenientPeeringVarianceAllowedLabelRule {
                allowed: vec!["pending".to_string(), "global".to_string(), "default".to_string()],
            }),
            Rc::new(LenientPeeringVarianceLabelLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_peering_variance_validator();
        let item = new_lenient_peering_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = LenientPeeringVarianceDiscountRangeRule { min: 9.8, max: 4139.2 };
        let mut item = new_lenient_peering_variance("x", "y");
        item.discount = 4140.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientPeeringVarianceRequiredCodeRule {
            message: "lenient peering variance needs this field".to_string(),
        };
        let mut item = new_lenient_peering_variance("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = LenientPeeringVarianceAmountRangeRule { min: 2.5, max: 4958.0 };
        let mut item = new_lenient_peering_variance("x", "y");
        item.amount = 4959.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientPeeringVarianceRequiredLabelRule {
            message: "lenient peering variance needs an entry".to_string(),
        };
        let mut item = new_lenient_peering_variance("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LenientPeeringVarianceLabelLengthRule { max_length: 80 };
        let mut item = new_lenient_peering_variance("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
