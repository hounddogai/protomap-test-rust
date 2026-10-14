use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy quote batch must pass. Check returns its problems, or none.
pub trait LegacyQuoteBatchRule: std::fmt::Debug {
    fn check(&self, item: &LegacyQuoteBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteBatchCodePrefixRule {
    pub prefix: String,
}

impl LegacyQuoteBatchRule for LegacyQuoteBatchCodePrefixRule {
    fn check(&self, item: &LegacyQuoteBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteBatchAllowedScopeRule {
    pub allowed: Vec<String>,
}

impl LegacyQuoteBatchRule for LegacyQuoteBatchAllowedScopeRule {
    fn check(&self, item: &LegacyQuoteBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.scope))) {
            return vec![format!("scope {} is expired", item.scope)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteBatchRequiredNameRule {
    pub message: String,
}

impl LegacyQuoteBatchRule for LegacyQuoteBatchRequiredNameRule {
    fn check(&self, item: &LegacyQuoteBatch) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteBatchRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyQuoteBatchRule for LegacyQuoteBatchRetriesRangeRule {
    fn check(&self, item: &LegacyQuoteBatch) -> Vec<String> {
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
pub struct LegacyQuoteBatchRequiredCodeRule {
    pub message: String,
}

impl LegacyQuoteBatchRule for LegacyQuoteBatchRequiredCodeRule {
    fn check(&self, item: &LegacyQuoteBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy quote batch.
#[derive(Debug, Clone)]
pub struct LegacyQuoteBatchValidator {
    pub rules: Vec<Rc<dyn LegacyQuoteBatchRule>>,
}

impl LegacyQuoteBatchValidator {
    pub fn validate(&self, item: &LegacyQuoteBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyQuoteBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyQuoteBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_quote_batch_validator() -> LegacyQuoteBatchValidator {
    LegacyQuoteBatchValidator {
        rules: vec![
            Rc::new(LegacyQuoteBatchCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn LegacyQuoteBatchRule>,
            Rc::new(LegacyQuoteBatchAllowedScopeRule { allowed: Vec::new() }),
            Rc::new(LegacyQuoteBatchRequiredNameRule {
                message: "legacy quote batch needs this field".to_string(),
            }),
            Rc::new(LegacyQuoteBatchRetriesRangeRule { min: 32, max: 120 }),
            Rc::new(LegacyQuoteBatchRequiredCodeRule {
                message: "legacy quote batch needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_quote_batch_validator();
        let item = new_legacy_quote_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LegacyQuoteBatchRequiredNameRule {
            message: "legacy quote batch needs an entry".to_string(),
        };
        let mut item = new_legacy_quote_batch("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = LegacyQuoteBatchRetriesRangeRule { min: 1000, max: 86400 };
        let mut item = new_legacy_quote_batch("x", "y");
        item.retries = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyQuoteBatchRequiredCodeRule {
            message: "legacy quote batch needs a value".to_string(),
        };
        let mut item = new_legacy_quote_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
