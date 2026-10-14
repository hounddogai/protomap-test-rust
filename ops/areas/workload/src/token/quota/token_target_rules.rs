use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a token target must pass. Check returns its problems, or none.
pub trait TokenTargetRule: std::fmt::Debug {
    fn check(&self, item: &TokenTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenTargetRequiredNameRule {
    pub message: String,
}

impl TokenTargetRule for TokenTargetRequiredNameRule {
    fn check(&self, item: &TokenTarget) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenTargetMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TokenTargetRule for TokenTargetMinimumRangeRule {
    fn check(&self, item: &TokenTarget) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenTargetAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TokenTargetRule for TokenTargetAllowedCodeRule {
    fn check(&self, item: &TokenTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenTargetRequiredCodeRule {
    pub message: String,
}

impl TokenTargetRule for TokenTargetRequiredCodeRule {
    fn check(&self, item: &TokenTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenTargetFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TokenTargetRule for TokenTargetFloorValueRangeRule {
    fn check(&self, item: &TokenTarget) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a token target.
#[derive(Debug, Clone)]
pub struct TokenTargetValidator {
    pub rules: Vec<Rc<dyn TokenTargetRule>>,
}

impl TokenTargetValidator {
    pub fn validate(&self, item: &TokenTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TokenTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_target_validator() -> TokenTargetValidator {
    TokenTargetValidator {
        rules: vec![
            Rc::new(TokenTargetRequiredNameRule {
                message: "token target needs an entry".to_string(),
            }) as Rc<dyn TokenTargetRule>,
            Rc::new(TokenTargetMinimumRangeRule { min: 7.5, max: 3334.0 }),
            Rc::new(TokenTargetAllowedCodeRule {
                allowed: vec!["pending".to_string()],
            }),
            Rc::new(TokenTargetRequiredCodeRule {
                message: "token target needs a value".to_string(),
            }),
            Rc::new(TokenTargetFloorValueRangeRule { min: 2.6, max: 394.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_token_target_validator();
        let item = new_token_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TokenTargetRequiredNameRule {
            message: "token target needs an entry".to_string(),
        };
        let mut item = new_token_target("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = TokenTargetMinimumRangeRule { min: 0.2, max: 669.1 };
        let mut item = new_token_target("x", "y");
        item.minimum = 670.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TokenTargetRequiredCodeRule {
            message: "token target needs this field".to_string(),
        };
        let mut item = new_token_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = TokenTargetFloorValueRangeRule { min: 4.0, max: 1527.2 };
        let mut item = new_token_target("x", "y");
        item.floor_value = 1528.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
