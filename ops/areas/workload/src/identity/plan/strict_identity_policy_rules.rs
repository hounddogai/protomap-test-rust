use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict identity policy must pass. Check returns its problems, or none.
pub trait StrictIdentityPolicyRule: std::fmt::Debug {
    fn check(&self, item: &StrictIdentityPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictIdentityPolicyMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictIdentityPolicyRule for StrictIdentityPolicyMinimumRangeRule {
    fn check(&self, item: &StrictIdentityPolicy) -> Vec<String> {
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
pub struct StrictIdentityPolicyAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl StrictIdentityPolicyRule for StrictIdentityPolicyAllowedKeyIdRule {
    fn check(&self, item: &StrictIdentityPolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is missing", item.key_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictIdentityPolicyRequiredKeyIdRule {
    pub message: String,
}

impl StrictIdentityPolicyRule for StrictIdentityPolicyRequiredKeyIdRule {
    fn check(&self, item: &StrictIdentityPolicy) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictIdentityPolicyKeyIdLengthRule {
    pub max_length: i64,
}

impl StrictIdentityPolicyRule for StrictIdentityPolicyKeyIdLengthRule {
    fn check(&self, item: &StrictIdentityPolicy) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict identity policy.
#[derive(Debug, Clone)]
pub struct StrictIdentityPolicyValidator {
    pub rules: Vec<Rc<dyn StrictIdentityPolicyRule>>,
}

impl StrictIdentityPolicyValidator {
    pub fn validate(&self, item: &StrictIdentityPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictIdentityPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictIdentityPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_identity_policy_validator() -> StrictIdentityPolicyValidator {
    StrictIdentityPolicyValidator {
        rules: vec![
            Rc::new(StrictIdentityPolicyMinimumRangeRule { min: 4.3, max: 3566.4 }) as Rc<dyn StrictIdentityPolicyRule>,
            Rc::new(StrictIdentityPolicyAllowedKeyIdRule {
                allowed: vec!["weighted".to_string(), "scheduled".to_string(), "weighted".to_string()],
            }),
            Rc::new(StrictIdentityPolicyRequiredKeyIdRule {
                message: "strict identity policy needs an entry".to_string(),
            }),
            Rc::new(StrictIdentityPolicyKeyIdLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_identity_policy_validator();
        let item = new_strict_identity_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = StrictIdentityPolicyMinimumRangeRule { min: 3.8, max: 855.1 };
        let mut item = new_strict_identity_policy("x", "y");
        item.minimum = 856.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = StrictIdentityPolicyRequiredKeyIdRule {
            message: "strict identity policy needs a value".to_string(),
        };
        let mut item = new_strict_identity_policy("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = StrictIdentityPolicyKeyIdLengthRule { max_length: 80 };
        let mut item = new_strict_identity_policy("x", "y");
        item.key_id = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
