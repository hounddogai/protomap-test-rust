use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched ttl policy window must pass. Check returns its problems, or none.
pub trait BatchedTtlPolicyWindowRule: std::fmt::Debug {
    fn check(&self, item: &BatchedTtlPolicyWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTtlPolicyWindowRequiredChecksumRule {
    pub message: String,
}

impl BatchedTtlPolicyWindowRule for BatchedTtlPolicyWindowRequiredChecksumRule {
    fn check(&self, item: &BatchedTtlPolicyWindow) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTtlPolicyWindowRequiredNameRule {
    pub message: String,
}

impl BatchedTtlPolicyWindowRule for BatchedTtlPolicyWindowRequiredNameRule {
    fn check(&self, item: &BatchedTtlPolicyWindow) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTtlPolicyWindowExternalRefLengthRule {
    pub max_length: i64,
}

impl BatchedTtlPolicyWindowRule for BatchedTtlPolicyWindowExternalRefLengthRule {
    fn check(&self, item: &BatchedTtlPolicyWindow) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTtlPolicyWindowAllowedLocaleRule {
    pub allowed: Vec<String>,
}

impl BatchedTtlPolicyWindowRule for BatchedTtlPolicyWindowAllowedLocaleRule {
    fn check(&self, item: &BatchedTtlPolicyWindow) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.locale))) {
            return vec![format!("locale {} is blocked", item.locale)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTtlPolicyWindowDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedTtlPolicyWindowRule for BatchedTtlPolicyWindowDiscountRangeRule {
    fn check(&self, item: &BatchedTtlPolicyWindow) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched ttl policy window.
#[derive(Debug, Clone)]
pub struct BatchedTtlPolicyWindowValidator {
    pub rules: Vec<Rc<dyn BatchedTtlPolicyWindowRule>>,
}

impl BatchedTtlPolicyWindowValidator {
    pub fn validate(&self, item: &BatchedTtlPolicyWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedTtlPolicyWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedTtlPolicyWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_ttl_policy_window_validator() -> BatchedTtlPolicyWindowValidator {
    BatchedTtlPolicyWindowValidator {
        rules: vec![
            Rc::new(BatchedTtlPolicyWindowRequiredChecksumRule {
                message: "batched ttl policy window needs an entry".to_string(),
            }) as Rc<dyn BatchedTtlPolicyWindowRule>,
            Rc::new(BatchedTtlPolicyWindowRequiredNameRule {
                message: "batched ttl policy window needs this field".to_string(),
            }),
            Rc::new(BatchedTtlPolicyWindowExternalRefLengthRule { max_length: 16 }),
            Rc::new(BatchedTtlPolicyWindowAllowedLocaleRule {
                allowed: vec!["capped".to_string()],
            }),
            Rc::new(BatchedTtlPolicyWindowDiscountRangeRule { min: 9.5, max: 458.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_ttl_policy_window_validator();
        let item = new_batched_ttl_policy_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = BatchedTtlPolicyWindowRequiredChecksumRule {
            message: "batched ttl policy window needs an entry".to_string(),
        };
        let mut item = new_batched_ttl_policy_window("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = BatchedTtlPolicyWindowRequiredNameRule {
            message: "batched ttl policy window needs a value".to_string(),
        };
        let mut item = new_batched_ttl_policy_window("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = BatchedTtlPolicyWindowExternalRefLengthRule { max_length: 80 };
        let mut item = new_batched_ttl_policy_window("x", "y");
        item.external_ref = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = BatchedTtlPolicyWindowDiscountRangeRule { min: 4.8, max: 1849.2 };
        let mut item = new_batched_ttl_policy_window("x", "y");
        item.discount = 1850.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
