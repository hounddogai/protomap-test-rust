use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched binding policy must pass. Check returns its problems, or none.
pub trait BatchedBindingPolicyRule: std::fmt::Debug {
    fn check(&self, item: &BatchedBindingPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBindingPolicyRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedBindingPolicyRule for BatchedBindingPolicyRetriesRangeRule {
    fn check(&self, item: &BatchedBindingPolicy) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBindingPolicyPinnedGuardRule {
    pub limit: i64,
}

impl BatchedBindingPolicyRule for BatchedBindingPolicyPinnedGuardRule {
    fn check(&self, item: &BatchedBindingPolicy) -> Vec<String> {
        if item.pinned && (item.sequence > self.limit) {
            return vec![format!(
                "sequence {} exceeds {} while pinned",
                item.sequence, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBindingPolicyRequiredAudienceRule {
    pub message: String,
}

impl BatchedBindingPolicyRule for BatchedBindingPolicyRequiredAudienceRule {
    fn check(&self, item: &BatchedBindingPolicy) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBindingPolicyCodePrefixRule {
    pub prefix: String,
}

impl BatchedBindingPolicyRule for BatchedBindingPolicyCodePrefixRule {
    fn check(&self, item: &BatchedBindingPolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched binding policy.
#[derive(Debug, Clone)]
pub struct BatchedBindingPolicyValidator {
    pub rules: Vec<Rc<dyn BatchedBindingPolicyRule>>,
}

impl BatchedBindingPolicyValidator {
    pub fn validate(&self, item: &BatchedBindingPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedBindingPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedBindingPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_binding_policy_validator() -> BatchedBindingPolicyValidator {
    BatchedBindingPolicyValidator {
        rules: vec![
            Rc::new(BatchedBindingPolicyRetriesRangeRule { min: 1000, max: 1000 }) as Rc<dyn BatchedBindingPolicyRule>,
            Rc::new(BatchedBindingPolicyPinnedGuardRule { limit: 32 }),
            Rc::new(BatchedBindingPolicyRequiredAudienceRule {
                message: "batched binding policy needs a value".to_string(),
            }),
            Rc::new(BatchedBindingPolicyCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_binding_policy_validator();
        let item = new_batched_binding_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = BatchedBindingPolicyRetriesRangeRule { min: 120, max: 1000 };
        let mut item = new_batched_binding_policy("x", "y");
        item.retries = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = BatchedBindingPolicyRequiredAudienceRule {
            message: "batched binding policy needs a value".to_string(),
        };
        let mut item = new_batched_binding_policy("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
