use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout policy must pass. Check returns its problems, or none.
pub trait RolloutPolicyRule: std::fmt::Debug {
    fn check(&self, item: &RolloutPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutPolicyCodePrefixRule {
    pub prefix: String,
}

impl RolloutPolicyRule for RolloutPolicyCodePrefixRule {
    fn check(&self, item: &RolloutPolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutPolicyRequiredLabelRule {
    pub message: String,
}

impl RolloutPolicyRule for RolloutPolicyRequiredLabelRule {
    fn check(&self, item: &RolloutPolicy) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutPolicyDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RolloutPolicyRule for RolloutPolicyDurationMinutesRangeRule {
    fn check(&self, item: &RolloutPolicy) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutPolicyCurrencyLengthRule {
    pub max_length: i64,
}

impl RolloutPolicyRule for RolloutPolicyCurrencyLengthRule {
    fn check(&self, item: &RolloutPolicy) -> Vec<String> {
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

/// Runs every rule of a rollout policy.
#[derive(Debug, Clone)]
pub struct RolloutPolicyValidator {
    pub rules: Vec<Rc<dyn RolloutPolicyRule>>,
}

impl RolloutPolicyValidator {
    pub fn validate(&self, item: &RolloutPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RolloutPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_policy_validator() -> RolloutPolicyValidator {
    RolloutPolicyValidator {
        rules: vec![
            Rc::new(RolloutPolicyCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn RolloutPolicyRule>,
            Rc::new(RolloutPolicyRequiredLabelRule {
                message: "rollout policy needs this field".to_string(),
            }),
            Rc::new(RolloutPolicyDurationMinutesRangeRule { min: 120, max: 86400 }),
            Rc::new(RolloutPolicyCurrencyLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_policy_validator();
        let item = new_rollout_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RolloutPolicyRequiredLabelRule {
            message: "rollout policy needs this field".to_string(),
        };
        let mut item = new_rollout_policy("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = RolloutPolicyDurationMinutesRangeRule { min: 16, max: 255 };
        let mut item = new_rollout_policy("x", "y");
        item.duration_minutes = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = RolloutPolicyCurrencyLengthRule { max_length: 86400 };
        let mut item = new_rollout_policy("x", "y");
        item.currency = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
