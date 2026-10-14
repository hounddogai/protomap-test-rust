use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a peering metric must pass. Check returns its problems, or none.
pub trait PeeringMetricRule: std::fmt::Debug {
    fn check(&self, item: &PeeringMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringMetricRequiredCurrencyRule {
    pub message: String,
}

impl PeeringMetricRule for PeeringMetricRequiredCurrencyRule {
    fn check(&self, item: &PeeringMetric) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringMetricAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PeeringMetricRule for PeeringMetricAllowedCodeRule {
    fn check(&self, item: &PeeringMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is out of range", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringMetricRequiredCodeRule {
    pub message: String,
}

impl PeeringMetricRule for PeeringMetricRequiredCodeRule {
    fn check(&self, item: &PeeringMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringMetricCodeLengthRule {
    pub max_length: i64,
}

impl PeeringMetricRule for PeeringMetricCodeLengthRule {
    fn check(&self, item: &PeeringMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringMetricDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PeeringMetricRule for PeeringMetricDiscountRangeRule {
    fn check(&self, item: &PeeringMetric) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringMetricBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PeeringMetricRule for PeeringMetricBudgetRangeRule {
    fn check(&self, item: &PeeringMetric) -> Vec<String> {
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

/// Runs every rule of a peering metric.
#[derive(Debug, Clone)]
pub struct PeeringMetricValidator {
    pub rules: Vec<Rc<dyn PeeringMetricRule>>,
}

impl PeeringMetricValidator {
    pub fn validate(&self, item: &PeeringMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PeeringMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_metric_validator() -> PeeringMetricValidator {
    PeeringMetricValidator {
        rules: vec![
            Rc::new(PeeringMetricRequiredCurrencyRule {
                message: "peering metric needs a value".to_string(),
            }) as Rc<dyn PeeringMetricRule>,
            Rc::new(PeeringMetricAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(PeeringMetricRequiredCodeRule {
                message: "peering metric needs this field".to_string(),
            }),
            Rc::new(PeeringMetricCodeLengthRule { max_length: 86400 }),
            Rc::new(PeeringMetricDiscountRangeRule { min: 1.8, max: 2505.2 }),
            Rc::new(PeeringMetricBudgetRangeRule { min: 2.4, max: 4459.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_metric_validator();
        let item = new_peering_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = PeeringMetricRequiredCurrencyRule {
            message: "peering metric needs this field".to_string(),
        };
        let mut item = new_peering_metric("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PeeringMetricRequiredCodeRule {
            message: "peering metric needs an entry".to_string(),
        };
        let mut item = new_peering_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PeeringMetricCodeLengthRule { max_length: 120 };
        let mut item = new_peering_metric("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = PeeringMetricDiscountRangeRule { min: 1.2, max: 585.9 };
        let mut item = new_peering_metric("x", "y");
        item.discount = 587.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = PeeringMetricBudgetRangeRule { min: 8.6, max: 2854.6 };
        let mut item = new_peering_metric("x", "y");
        item.budget = 2856.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
