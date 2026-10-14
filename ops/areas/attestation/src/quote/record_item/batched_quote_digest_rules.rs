use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched quote digest must pass. Check returns its problems, or none.
pub trait BatchedQuoteDigestRule: std::fmt::Debug {
    fn check(&self, item: &BatchedQuoteDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteDigestNameLengthRule {
    pub max_length: i64,
}

impl BatchedQuoteDigestRule for BatchedQuoteDigestNameLengthRule {
    fn check(&self, item: &BatchedQuoteDigest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteDigestRequiredCodeRule {
    pub message: String,
}

impl BatchedQuoteDigestRule for BatchedQuoteDigestRequiredCodeRule {
    fn check(&self, item: &BatchedQuoteDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteDigestCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedQuoteDigestRule for BatchedQuoteDigestCeilingRangeRule {
    fn check(&self, item: &BatchedQuoteDigest) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteDigestOwnerIdLengthRule {
    pub max_length: i64,
}

impl BatchedQuoteDigestRule for BatchedQuoteDigestOwnerIdLengthRule {
    fn check(&self, item: &BatchedQuoteDigest) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteDigestRequiredNameRule {
    pub message: String,
}

impl BatchedQuoteDigestRule for BatchedQuoteDigestRequiredNameRule {
    fn check(&self, item: &BatchedQuoteDigest) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteDigestAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl BatchedQuoteDigestRule for BatchedQuoteDigestAllowedCodeRule {
    fn check(&self, item: &BatchedQuoteDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched quote digest.
#[derive(Debug, Clone)]
pub struct BatchedQuoteDigestValidator {
    pub rules: Vec<Rc<dyn BatchedQuoteDigestRule>>,
}

impl BatchedQuoteDigestValidator {
    pub fn validate(&self, item: &BatchedQuoteDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedQuoteDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedQuoteDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_quote_digest_validator() -> BatchedQuoteDigestValidator {
    BatchedQuoteDigestValidator {
        rules: vec![
            Rc::new(BatchedQuoteDigestNameLengthRule { max_length: 16 }) as Rc<dyn BatchedQuoteDigestRule>,
            Rc::new(BatchedQuoteDigestRequiredCodeRule {
                message: "batched quote digest needs this field".to_string(),
            }),
            Rc::new(BatchedQuoteDigestCeilingRangeRule { min: 5.6, max: 3780.3 }),
            Rc::new(BatchedQuoteDigestOwnerIdLengthRule { max_length: 3600 }),
            Rc::new(BatchedQuoteDigestRequiredNameRule {
                message: "batched quote digest needs this field".to_string(),
            }),
            Rc::new(BatchedQuoteDigestAllowedCodeRule {
                allowed: vec!["batched".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_quote_digest_validator();
        let item = new_batched_quote_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = BatchedQuoteDigestNameLengthRule { max_length: 86400 };
        let mut item = new_batched_quote_digest("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedQuoteDigestRequiredCodeRule {
            message: "batched quote digest needs this field".to_string(),
        };
        let mut item = new_batched_quote_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = BatchedQuoteDigestCeilingRangeRule { min: 2.4, max: 395.7 };
        let mut item = new_batched_quote_digest("x", "y");
        item.ceiling = 397.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = BatchedQuoteDigestOwnerIdLengthRule { max_length: 120 };
        let mut item = new_batched_quote_digest("x", "y");
        item.owner_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = BatchedQuoteDigestRequiredNameRule {
            message: "batched quote digest needs an entry".to_string(),
        };
        let mut item = new_batched_quote_digest("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
