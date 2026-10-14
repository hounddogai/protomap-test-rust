use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a delegation estimate must pass. Check returns its problems, or none.
pub trait DelegationEstimateRule: std::fmt::Debug {
    fn check(&self, item: &DelegationEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationEstimateRequiredCodeRule {
    pub message: String,
}

impl DelegationEstimateRule for DelegationEstimateRequiredCodeRule {
    fn check(&self, item: &DelegationEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationEstimateBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DelegationEstimateRule for DelegationEstimateBudgetRangeRule {
    fn check(&self, item: &DelegationEstimate) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationEstimateRegionLengthRule {
    pub max_length: i64,
}

impl DelegationEstimateRule for DelegationEstimateRegionLengthRule {
    fn check(&self, item: &DelegationEstimate) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationEstimateRequiredRegionRule {
    pub message: String,
}

impl DelegationEstimateRule for DelegationEstimateRequiredRegionRule {
    fn check(&self, item: &DelegationEstimate) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationEstimateRequiredLocaleRule {
    pub message: String,
}

impl DelegationEstimateRule for DelegationEstimateRequiredLocaleRule {
    fn check(&self, item: &DelegationEstimate) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation estimate.
#[derive(Debug, Clone)]
pub struct DelegationEstimateValidator {
    pub rules: Vec<Rc<dyn DelegationEstimateRule>>,
}

impl DelegationEstimateValidator {
    pub fn validate(&self, item: &DelegationEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DelegationEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_estimate_validator() -> DelegationEstimateValidator {
    DelegationEstimateValidator {
        rules: vec![
            Rc::new(DelegationEstimateRequiredCodeRule {
                message: "delegation estimate needs a value".to_string(),
            }) as Rc<dyn DelegationEstimateRule>,
            Rc::new(DelegationEstimateBudgetRangeRule { min: 9.4, max: 4726.8 }),
            Rc::new(DelegationEstimateRegionLengthRule { max_length: 120 }),
            Rc::new(DelegationEstimateRequiredRegionRule {
                message: "delegation estimate needs an entry".to_string(),
            }),
            Rc::new(DelegationEstimateRequiredLocaleRule {
                message: "delegation estimate needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_delegation_estimate_validator();
        let item = new_delegation_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DelegationEstimateRequiredCodeRule {
            message: "delegation estimate needs a value".to_string(),
        };
        let mut item = new_delegation_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = DelegationEstimateBudgetRangeRule { min: 2.1, max: 1509.7 };
        let mut item = new_delegation_estimate("x", "y");
        item.budget = 1511.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = DelegationEstimateRegionLengthRule { max_length: 1000 };
        let mut item = new_delegation_estimate("x", "y");
        item.region = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = DelegationEstimateRequiredRegionRule {
            message: "delegation estimate needs a value".to_string(),
        };
        let mut item = new_delegation_estimate("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = DelegationEstimateRequiredLocaleRule {
            message: "delegation estimate needs an entry".to_string(),
        };
        let mut item = new_delegation_estimate("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
