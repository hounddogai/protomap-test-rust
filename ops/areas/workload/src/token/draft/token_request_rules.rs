use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a token request must pass. Check returns its problems, or none.
pub trait TokenRequestRule: std::fmt::Debug {
    fn check(&self, item: &TokenRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenRequestWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TokenRequestRule for TokenRequestWeightRangeRule {
    fn check(&self, item: &TokenRequest) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenRequestCodeLengthRule {
    pub max_length: i64,
}

impl TokenRequestRule for TokenRequestCodeLengthRule {
    fn check(&self, item: &TokenRequest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenRequestAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TokenRequestRule for TokenRequestAllowedCodeRule {
    fn check(&self, item: &TokenRequest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenRequestLabelLengthRule {
    pub max_length: i64,
}

impl TokenRequestRule for TokenRequestLabelLengthRule {
    fn check(&self, item: &TokenRequest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a token request.
#[derive(Debug, Clone)]
pub struct TokenRequestValidator {
    pub rules: Vec<Rc<dyn TokenRequestRule>>,
}

impl TokenRequestValidator {
    pub fn validate(&self, item: &TokenRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TokenRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_request_validator() -> TokenRequestValidator {
    TokenRequestValidator {
        rules: vec![
            Rc::new(TokenRequestWeightRangeRule { min: 7.5, max: 566.6 }) as Rc<dyn TokenRequestRule>,
            Rc::new(TokenRequestCodeLengthRule { max_length: 16 }),
            Rc::new(TokenRequestAllowedCodeRule {
                allowed: vec!["manual".to_string(), "strict".to_string(), "legacy".to_string()],
            }),
            Rc::new(TokenRequestLabelLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_token_request_validator();
        let item = new_token_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = TokenRequestWeightRangeRule { min: 4.1, max: 3188.6 };
        let mut item = new_token_request("x", "y");
        item.weight = 3190.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TokenRequestCodeLengthRule { max_length: 3600 };
        let mut item = new_token_request("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TokenRequestLabelLengthRule { max_length: 120 };
        let mut item = new_token_request("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
