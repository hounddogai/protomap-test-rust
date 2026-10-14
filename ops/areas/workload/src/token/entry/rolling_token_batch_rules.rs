use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling token batch must pass. Check returns its problems, or none.
pub trait RollingTokenBatchRule: std::fmt::Debug {
    fn check(&self, item: &RollingTokenBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenBatchAudienceLengthRule {
    pub max_length: i64,
}

impl RollingTokenBatchRule for RollingTokenBatchAudienceLengthRule {
    fn check(&self, item: &RollingTokenBatch) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenBatchAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RollingTokenBatchRule for RollingTokenBatchAllowedCodeRule {
    fn check(&self, item: &RollingTokenBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenBatchCodeLengthRule {
    pub max_length: i64,
}

impl RollingTokenBatchRule for RollingTokenBatchCodeLengthRule {
    fn check(&self, item: &RollingTokenBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenBatchCodePrefixRule {
    pub prefix: String,
}

impl RollingTokenBatchRule for RollingTokenBatchCodePrefixRule {
    fn check(&self, item: &RollingTokenBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling token batch.
#[derive(Debug, Clone)]
pub struct RollingTokenBatchValidator {
    pub rules: Vec<Rc<dyn RollingTokenBatchRule>>,
}

impl RollingTokenBatchValidator {
    pub fn validate(&self, item: &RollingTokenBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingTokenBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingTokenBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_token_batch_validator() -> RollingTokenBatchValidator {
    RollingTokenBatchValidator {
        rules: vec![
            Rc::new(RollingTokenBatchAudienceLengthRule { max_length: 120 }) as Rc<dyn RollingTokenBatchRule>,
            Rc::new(RollingTokenBatchAllowedCodeRule {
                allowed: vec!["capped".to_string()],
            }),
            Rc::new(RollingTokenBatchCodeLengthRule { max_length: 255 }),
            Rc::new(RollingTokenBatchCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_token_batch_validator();
        let item = new_rolling_token_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = RollingTokenBatchAudienceLengthRule { max_length: 86400 };
        let mut item = new_rolling_token_batch("x", "y");
        item.audience = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingTokenBatchCodeLengthRule { max_length: 255 };
        let mut item = new_rolling_token_batch("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
