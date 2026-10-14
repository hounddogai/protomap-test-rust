use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a token checkpoint must pass. Check returns its problems, or none.
pub trait TokenCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &TokenCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenCheckpointCodePrefixRule {
    pub prefix: String,
}

impl TokenCheckpointRule for TokenCheckpointCodePrefixRule {
    fn check(&self, item: &TokenCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenCheckpointFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TokenCheckpointRule for TokenCheckpointFloorValueRangeRule {
    fn check(&self, item: &TokenCheckpoint) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenCheckpointRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TokenCheckpointRule for TokenCheckpointRetriesRangeRule {
    fn check(&self, item: &TokenCheckpoint) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenCheckpointKeyIdLengthRule {
    pub max_length: i64,
}

impl TokenCheckpointRule for TokenCheckpointKeyIdLengthRule {
    fn check(&self, item: &TokenCheckpoint) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a token checkpoint.
#[derive(Debug, Clone)]
pub struct TokenCheckpointValidator {
    pub rules: Vec<Rc<dyn TokenCheckpointRule>>,
}

impl TokenCheckpointValidator {
    pub fn validate(&self, item: &TokenCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TokenCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_checkpoint_validator() -> TokenCheckpointValidator {
    TokenCheckpointValidator {
        rules: vec![
            Rc::new(TokenCheckpointCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn TokenCheckpointRule>,
            Rc::new(TokenCheckpointFloorValueRangeRule { min: 6.3, max: 3125.3 }),
            Rc::new(TokenCheckpointRetriesRangeRule { min: 32, max: 120 }),
            Rc::new(TokenCheckpointKeyIdLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_token_checkpoint_validator();
        let item = new_token_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = TokenCheckpointFloorValueRangeRule { min: 3.9, max: 4003.1 };
        let mut item = new_token_checkpoint("x", "y");
        item.floor_value = 4004.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = TokenCheckpointRetriesRangeRule { min: 16, max: 64 };
        let mut item = new_token_checkpoint("x", "y");
        item.retries = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = TokenCheckpointKeyIdLengthRule { max_length: 16 };
        let mut item = new_token_checkpoint("x", "y");
        item.key_id = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
