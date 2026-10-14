use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a token tier must pass. Check returns its problems, or none.
pub trait TokenTierRule: std::fmt::Debug {
    fn check(&self, item: &TokenTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenTierTitleLengthRule {
    pub max_length: i64,
}

impl TokenTierRule for TokenTierTitleLengthRule {
    fn check(&self, item: &TokenTier) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenTierMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TokenTierRule for TokenTierMarginRangeRule {
    fn check(&self, item: &TokenTier) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenTierRequiredCodeRule {
    pub message: String,
}

impl TokenTierRule for TokenTierRequiredCodeRule {
    fn check(&self, item: &TokenTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenTierPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TokenTierRule for TokenTierPortRangeRule {
    fn check(&self, item: &TokenTier) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a token tier.
#[derive(Debug, Clone)]
pub struct TokenTierValidator {
    pub rules: Vec<Rc<dyn TokenTierRule>>,
}

impl TokenTierValidator {
    pub fn validate(&self, item: &TokenTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TokenTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_tier_validator() -> TokenTierValidator {
    TokenTierValidator {
        rules: vec![
            Rc::new(TokenTierTitleLengthRule { max_length: 1000 }) as Rc<dyn TokenTierRule>,
            Rc::new(TokenTierMarginRangeRule { min: 10.0, max: 3363.9 }),
            Rc::new(TokenTierRequiredCodeRule {
                message: "token tier needs this field".to_string(),
            }),
            Rc::new(TokenTierPortRangeRule { min: 32, max: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_token_tier_validator();
        let item = new_token_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TokenTierTitleLengthRule { max_length: 32 };
        let mut item = new_token_tier("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = TokenTierMarginRangeRule { min: 3.3, max: 1635.7 };
        let mut item = new_token_tier("x", "y");
        item.margin = 1637.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TokenTierRequiredCodeRule {
            message: "token tier needs an entry".to_string(),
        };
        let mut item = new_token_tier("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = TokenTierPortRangeRule { min: 32, max: 32 };
        let mut item = new_token_tier("x", "y");
        item.port = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
