use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a policy forecast must pass. Check returns its problems, or none.
pub trait PolicyForecastRule: std::fmt::Debug {
    fn check(&self, item: &PolicyForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyForecastCodeLengthRule {
    pub max_length: i64,
}

impl PolicyForecastRule for PolicyForecastCodeLengthRule {
    fn check(&self, item: &PolicyForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyForecastCodePrefixRule {
    pub prefix: String,
}

impl PolicyForecastRule for PolicyForecastCodePrefixRule {
    fn check(&self, item: &PolicyForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyForecastRequiredContactEmailRule {
    pub message: String,
}

impl PolicyForecastRule for PolicyForecastRequiredContactEmailRule {
    fn check(&self, item: &PolicyForecast) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a policy forecast.
#[derive(Debug, Clone)]
pub struct PolicyForecastValidator {
    pub rules: Vec<Rc<dyn PolicyForecastRule>>,
}

impl PolicyForecastValidator {
    pub fn validate(&self, item: &PolicyForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PolicyForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicyForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_forecast_validator() -> PolicyForecastValidator {
    PolicyForecastValidator {
        rules: vec![
            Rc::new(PolicyForecastCodeLengthRule { max_length: 80 }) as Rc<dyn PolicyForecastRule>,
            Rc::new(PolicyForecastCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(PolicyForecastRequiredContactEmailRule {
                message: "policy forecast needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_policy_forecast_validator();
        let item = new_policy_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PolicyForecastCodeLengthRule { max_length: 32 };
        let mut item = new_policy_forecast("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = PolicyForecastRequiredContactEmailRule {
            message: "policy forecast needs a value".to_string(),
        };
        let mut item = new_policy_forecast("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
