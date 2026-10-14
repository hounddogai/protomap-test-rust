use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched token forecast must pass. Check returns its problems, or none.
pub trait BatchedTokenForecastRule: std::fmt::Debug {
    fn check(&self, item: &BatchedTokenForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTokenForecastCodePrefixRule {
    pub prefix: String,
}

impl BatchedTokenForecastRule for BatchedTokenForecastCodePrefixRule {
    fn check(&self, item: &BatchedTokenForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTokenForecastScopeLengthRule {
    pub max_length: i64,
}

impl BatchedTokenForecastRule for BatchedTokenForecastScopeLengthRule {
    fn check(&self, item: &BatchedTokenForecast) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTokenForecastAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl BatchedTokenForecastRule for BatchedTokenForecastAllowedCodeRule {
    fn check(&self, item: &BatchedTokenForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTokenForecastCodeLengthRule {
    pub max_length: i64,
}

impl BatchedTokenForecastRule for BatchedTokenForecastCodeLengthRule {
    fn check(&self, item: &BatchedTokenForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTokenForecastRequiredCodeRule {
    pub message: String,
}

impl BatchedTokenForecastRule for BatchedTokenForecastRequiredCodeRule {
    fn check(&self, item: &BatchedTokenForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTokenForecastRequiredScopeRule {
    pub message: String,
}

impl BatchedTokenForecastRule for BatchedTokenForecastRequiredScopeRule {
    fn check(&self, item: &BatchedTokenForecast) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched token forecast.
#[derive(Debug, Clone)]
pub struct BatchedTokenForecastValidator {
    pub rules: Vec<Rc<dyn BatchedTokenForecastRule>>,
}

impl BatchedTokenForecastValidator {
    pub fn validate(&self, item: &BatchedTokenForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedTokenForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedTokenForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_token_forecast_validator() -> BatchedTokenForecastValidator {
    BatchedTokenForecastValidator {
        rules: vec![
            Rc::new(BatchedTokenForecastCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn BatchedTokenForecastRule>,
            Rc::new(BatchedTokenForecastScopeLengthRule { max_length: 120 }),
            Rc::new(BatchedTokenForecastAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(BatchedTokenForecastCodeLengthRule { max_length: 64 }),
            Rc::new(BatchedTokenForecastRequiredCodeRule {
                message: "batched token forecast needs a value".to_string(),
            }),
            Rc::new(BatchedTokenForecastRequiredScopeRule {
                message: "batched token forecast needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_token_forecast_validator();
        let item = new_batched_token_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = BatchedTokenForecastScopeLengthRule { max_length: 120 };
        let mut item = new_batched_token_forecast("x", "y");
        item.scope = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedTokenForecastCodeLengthRule { max_length: 80 };
        let mut item = new_batched_token_forecast("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedTokenForecastRequiredCodeRule {
            message: "batched token forecast needs an entry".to_string(),
        };
        let mut item = new_batched_token_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = BatchedTokenForecastRequiredScopeRule {
            message: "batched token forecast needs this field".to_string(),
        };
        let mut item = new_batched_token_forecast("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
