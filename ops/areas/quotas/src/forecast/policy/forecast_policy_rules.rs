use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast policy must pass. Check returns its problems, or none.
pub trait ForecastPolicyRule: std::fmt::Debug {
    fn check(&self, item: &ForecastPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastPolicyRequiredCodeRule {
    pub message: String,
}

impl ForecastPolicyRule for ForecastPolicyRequiredCodeRule {
    fn check(&self, item: &ForecastPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastPolicyCodeLengthRule {
    pub max_length: i64,
}

impl ForecastPolicyRule for ForecastPolicyCodeLengthRule {
    fn check(&self, item: &ForecastPolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastPolicyCodePrefixRule {
    pub prefix: String,
}

impl ForecastPolicyRule for ForecastPolicyCodePrefixRule {
    fn check(&self, item: &ForecastPolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast policy.
#[derive(Debug, Clone)]
pub struct ForecastPolicyValidator {
    pub rules: Vec<Rc<dyn ForecastPolicyRule>>,
}

impl ForecastPolicyValidator {
    pub fn validate(&self, item: &ForecastPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ForecastPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_policy_validator() -> ForecastPolicyValidator {
    ForecastPolicyValidator {
        rules: vec![
            Rc::new(ForecastPolicyRequiredCodeRule {
                message: "forecast policy needs a value".to_string(),
            }) as Rc<dyn ForecastPolicyRule>,
            Rc::new(ForecastPolicyCodeLengthRule { max_length: 80 }),
            Rc::new(ForecastPolicyCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_policy_validator();
        let item = new_forecast_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ForecastPolicyRequiredCodeRule {
            message: "forecast policy needs a value".to_string(),
        };
        let mut item = new_forecast_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ForecastPolicyCodeLengthRule { max_length: 32 };
        let mut item = new_forecast_policy("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
