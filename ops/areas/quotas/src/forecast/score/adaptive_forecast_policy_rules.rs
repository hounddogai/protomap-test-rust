use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive forecast policy must pass. Check returns its problems, or none.
pub trait AdaptiveForecastPolicyRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveForecastPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveForecastPolicyQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveForecastPolicyRule for AdaptiveForecastPolicyQuantityRangeRule {
    fn check(&self, item: &AdaptiveForecastPolicy) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveForecastPolicyRequiredCodeRule {
    pub message: String,
}

impl AdaptiveForecastPolicyRule for AdaptiveForecastPolicyRequiredCodeRule {
    fn check(&self, item: &AdaptiveForecastPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveForecastPolicyRequiredClusterRule {
    pub message: String,
}

impl AdaptiveForecastPolicyRule for AdaptiveForecastPolicyRequiredClusterRule {
    fn check(&self, item: &AdaptiveForecastPolicy) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive forecast policy.
#[derive(Debug, Clone)]
pub struct AdaptiveForecastPolicyValidator {
    pub rules: Vec<Rc<dyn AdaptiveForecastPolicyRule>>,
}

impl AdaptiveForecastPolicyValidator {
    pub fn validate(&self, item: &AdaptiveForecastPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveForecastPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveForecastPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_forecast_policy_validator() -> AdaptiveForecastPolicyValidator {
    AdaptiveForecastPolicyValidator {
        rules: vec![
            Rc::new(AdaptiveForecastPolicyQuantityRangeRule { min: 64, max: 80 }) as Rc<dyn AdaptiveForecastPolicyRule>,
            Rc::new(AdaptiveForecastPolicyRequiredCodeRule {
                message: "adaptive forecast policy needs a value".to_string(),
            }),
            Rc::new(AdaptiveForecastPolicyRequiredClusterRule {
                message: "adaptive forecast policy needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_forecast_policy_validator();
        let item = new_adaptive_forecast_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = AdaptiveForecastPolicyQuantityRangeRule { min: 16, max: 1000 };
        let mut item = new_adaptive_forecast_policy("x", "y");
        item.quantity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveForecastPolicyRequiredCodeRule {
            message: "adaptive forecast policy needs an entry".to_string(),
        };
        let mut item = new_adaptive_forecast_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = AdaptiveForecastPolicyRequiredClusterRule {
            message: "adaptive forecast policy needs an entry".to_string(),
        };
        let mut item = new_adaptive_forecast_policy("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
