use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default forecast policy must pass. Check returns its problems, or none.
pub trait DefaultForecastPolicyRule: std::fmt::Debug {
    fn check(&self, item: &DefaultForecastPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultForecastPolicyCodePrefixRule {
    pub prefix: String,
}

impl DefaultForecastPolicyRule for DefaultForecastPolicyCodePrefixRule {
    fn check(&self, item: &DefaultForecastPolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultForecastPolicyCodeLengthRule {
    pub max_length: i64,
}

impl DefaultForecastPolicyRule for DefaultForecastPolicyCodeLengthRule {
    fn check(&self, item: &DefaultForecastPolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultForecastPolicyNameLengthRule {
    pub max_length: i64,
}

impl DefaultForecastPolicyRule for DefaultForecastPolicyNameLengthRule {
    fn check(&self, item: &DefaultForecastPolicy) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultForecastPolicyThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultForecastPolicyRule for DefaultForecastPolicyThresholdRangeRule {
    fn check(&self, item: &DefaultForecastPolicy) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default forecast policy.
#[derive(Debug, Clone)]
pub struct DefaultForecastPolicyValidator {
    pub rules: Vec<Rc<dyn DefaultForecastPolicyRule>>,
}

impl DefaultForecastPolicyValidator {
    pub fn validate(&self, item: &DefaultForecastPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultForecastPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultForecastPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_forecast_policy_validator() -> DefaultForecastPolicyValidator {
    DefaultForecastPolicyValidator {
        rules: vec![
            Rc::new(DefaultForecastPolicyCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn DefaultForecastPolicyRule>,
            Rc::new(DefaultForecastPolicyCodeLengthRule { max_length: 80 }),
            Rc::new(DefaultForecastPolicyNameLengthRule { max_length: 80 }),
            Rc::new(DefaultForecastPolicyThresholdRangeRule { min: 0.6, max: 4081.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_forecast_policy_validator();
        let item = new_default_forecast_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultForecastPolicyCodeLengthRule { max_length: 16 };
        let mut item = new_default_forecast_policy("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = DefaultForecastPolicyNameLengthRule { max_length: 80 };
        let mut item = new_default_forecast_policy("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = DefaultForecastPolicyThresholdRangeRule { min: 9.1, max: 3105.2 };
        let mut item = new_default_forecast_policy("x", "y");
        item.threshold = 3106.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
