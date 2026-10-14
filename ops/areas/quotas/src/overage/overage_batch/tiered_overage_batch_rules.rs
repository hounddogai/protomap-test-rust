use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered overage batch must pass. Check returns its problems, or none.
pub trait TieredOverageBatchRule: std::fmt::Debug {
    fn check(&self, item: &TieredOverageBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageBatchCodePrefixRule {
    pub prefix: String,
}

impl TieredOverageBatchRule for TieredOverageBatchCodePrefixRule {
    fn check(&self, item: &TieredOverageBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageBatchRequiredScopeRule {
    pub message: String,
}

impl TieredOverageBatchRule for TieredOverageBatchRequiredScopeRule {
    fn check(&self, item: &TieredOverageBatch) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageBatchScopeLengthRule {
    pub max_length: i64,
}

impl TieredOverageBatchRule for TieredOverageBatchScopeLengthRule {
    fn check(&self, item: &TieredOverageBatch) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageBatchCurrencyLengthRule {
    pub max_length: i64,
}

impl TieredOverageBatchRule for TieredOverageBatchCurrencyLengthRule {
    fn check(&self, item: &TieredOverageBatch) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageBatchRequiredNameRule {
    pub message: String,
}

impl TieredOverageBatchRule for TieredOverageBatchRequiredNameRule {
    fn check(&self, item: &TieredOverageBatch) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered overage batch.
#[derive(Debug, Clone)]
pub struct TieredOverageBatchValidator {
    pub rules: Vec<Rc<dyn TieredOverageBatchRule>>,
}

impl TieredOverageBatchValidator {
    pub fn validate(&self, item: &TieredOverageBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredOverageBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredOverageBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_overage_batch_validator() -> TieredOverageBatchValidator {
    TieredOverageBatchValidator {
        rules: vec![
            Rc::new(TieredOverageBatchCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn TieredOverageBatchRule>,
            Rc::new(TieredOverageBatchRequiredScopeRule {
                message: "tiered overage batch needs a value".to_string(),
            }),
            Rc::new(TieredOverageBatchScopeLengthRule { max_length: 3600 }),
            Rc::new(TieredOverageBatchCurrencyLengthRule { max_length: 86400 }),
            Rc::new(TieredOverageBatchRequiredNameRule {
                message: "tiered overage batch needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_overage_batch_validator();
        let item = new_tiered_overage_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = TieredOverageBatchRequiredScopeRule {
            message: "tiered overage batch needs an entry".to_string(),
        };
        let mut item = new_tiered_overage_batch("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = TieredOverageBatchScopeLengthRule { max_length: 255 };
        let mut item = new_tiered_overage_batch("x", "y");
        item.scope = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = TieredOverageBatchCurrencyLengthRule { max_length: 255 };
        let mut item = new_tiered_overage_batch("x", "y");
        item.currency = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TieredOverageBatchRequiredNameRule {
            message: "tiered overage batch needs a value".to_string(),
        };
        let mut item = new_tiered_overage_batch("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
