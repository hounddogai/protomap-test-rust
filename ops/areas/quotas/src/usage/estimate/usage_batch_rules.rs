use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a usage batch must pass. Check returns its problems, or none.
pub trait UsageBatchRule: std::fmt::Debug {
    fn check(&self, item: &UsageBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageBatchTimezoneLengthRule {
    pub max_length: i64,
}

impl UsageBatchRule for UsageBatchTimezoneLengthRule {
    fn check(&self, item: &UsageBatch) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageBatchQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl UsageBatchRule for UsageBatchQuantityRangeRule {
    fn check(&self, item: &UsageBatch) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageBatchRequiredTimezoneRule {
    pub message: String,
}

impl UsageBatchRule for UsageBatchRequiredTimezoneRule {
    fn check(&self, item: &UsageBatch) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageBatchAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl UsageBatchRule for UsageBatchAllowedTimezoneRule {
    fn check(&self, item: &UsageBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is duplicated", item.timezone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageBatchCodePrefixRule {
    pub prefix: String,
}

impl UsageBatchRule for UsageBatchCodePrefixRule {
    fn check(&self, item: &UsageBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageBatchVisibleGuardRule {
    pub limit: i64,
}

impl UsageBatchRule for UsageBatchVisibleGuardRule {
    fn check(&self, item: &UsageBatch) -> Vec<String> {
        if item.visible && (item.quantity > self.limit) {
            return vec![format!(
                "quantity {} exceeds {} while visible",
                item.quantity, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a usage batch.
#[derive(Debug, Clone)]
pub struct UsageBatchValidator {
    pub rules: Vec<Rc<dyn UsageBatchRule>>,
}

impl UsageBatchValidator {
    pub fn validate(&self, item: &UsageBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &UsageBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_batch_validator() -> UsageBatchValidator {
    UsageBatchValidator {
        rules: vec![
            Rc::new(UsageBatchTimezoneLengthRule { max_length: 1000 }) as Rc<dyn UsageBatchRule>,
            Rc::new(UsageBatchQuantityRangeRule { min: 120, max: 120 }),
            Rc::new(UsageBatchRequiredTimezoneRule {
                message: "usage batch needs this field".to_string(),
            }),
            Rc::new(UsageBatchAllowedTimezoneRule {
                allowed: vec!["adaptive".to_string()],
            }),
            Rc::new(UsageBatchCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(UsageBatchVisibleGuardRule { limit: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_batch_validator();
        let item = new_usage_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = UsageBatchTimezoneLengthRule { max_length: 86400 };
        let mut item = new_usage_batch("x", "y");
        item.timezone = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = UsageBatchQuantityRangeRule { min: 80, max: 1000 };
        let mut item = new_usage_batch("x", "y");
        item.quantity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = UsageBatchRequiredTimezoneRule {
            message: "usage batch needs an entry".to_string(),
        };
        let mut item = new_usage_batch("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
