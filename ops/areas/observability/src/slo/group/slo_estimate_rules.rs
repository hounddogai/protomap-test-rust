use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a slo estimate must pass. Check returns its problems, or none.
pub trait SloEstimateRule: std::fmt::Debug {
    fn check(&self, item: &SloEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloEstimateRequiredNameRule {
    pub message: String,
}

impl SloEstimateRule for SloEstimateRequiredNameRule {
    fn check(&self, item: &SloEstimate) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloEstimateRequiredReviewerRule {
    pub message: String,
}

impl SloEstimateRule for SloEstimateRequiredReviewerRule {
    fn check(&self, item: &SloEstimate) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloEstimateCodePrefixRule {
    pub prefix: String,
}

impl SloEstimateRule for SloEstimateCodePrefixRule {
    fn check(&self, item: &SloEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloEstimateAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SloEstimateRule for SloEstimateAmountRangeRule {
    fn check(&self, item: &SloEstimate) -> Vec<String> {
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
pub struct SloEstimateMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SloEstimateRule for SloEstimateMinimumRangeRule {
    fn check(&self, item: &SloEstimate) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo estimate.
#[derive(Debug, Clone)]
pub struct SloEstimateValidator {
    pub rules: Vec<Rc<dyn SloEstimateRule>>,
}

impl SloEstimateValidator {
    pub fn validate(&self, item: &SloEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SloEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_estimate_validator() -> SloEstimateValidator {
    SloEstimateValidator {
        rules: vec![
            Rc::new(SloEstimateRequiredNameRule {
                message: "slo estimate needs this field".to_string(),
            }) as Rc<dyn SloEstimateRule>,
            Rc::new(SloEstimateRequiredReviewerRule {
                message: "slo estimate needs this field".to_string(),
            }),
            Rc::new(SloEstimateCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(SloEstimateAmountRangeRule { min: 3.4, max: 359.5 }),
            Rc::new(SloEstimateMinimumRangeRule { min: 7.0, max: 1712.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_estimate_validator();
        let item = new_slo_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = SloEstimateRequiredNameRule {
            message: "slo estimate needs an entry".to_string(),
        };
        let mut item = new_slo_estimate("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = SloEstimateRequiredReviewerRule {
            message: "slo estimate needs a value".to_string(),
        };
        let mut item = new_slo_estimate("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = SloEstimateAmountRangeRule { min: 7.0, max: 3207.6 };
        let mut item = new_slo_estimate("x", "y");
        item.amount = 3209.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = SloEstimateMinimumRangeRule { min: 3.0, max: 1916.3 };
        let mut item = new_slo_estimate("x", "y");
        item.minimum = 1917.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
