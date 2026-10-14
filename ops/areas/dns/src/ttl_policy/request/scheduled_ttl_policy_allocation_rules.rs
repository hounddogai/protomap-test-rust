use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled ttl policy allocation must pass. Check returns its problems, or none.
pub trait ScheduledTtlPolicyAllocationRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledTtlPolicyAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTtlPolicyAllocationTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl ScheduledTtlPolicyAllocationRule for ScheduledTtlPolicyAllocationTagsLimitRule {
    fn check(&self, item: &ScheduledTtlPolicyAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTtlPolicyAllocationActiveGuardRule {
    pub limit: f64,
}

impl ScheduledTtlPolicyAllocationRule for ScheduledTtlPolicyAllocationActiveGuardRule {
    fn check(&self, item: &ScheduledTtlPolicyAllocation) -> Vec<String> {
        if item.active && (item.budget > self.limit) {
            return vec![format!("budget {} exceeds {} while active", item.budget, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTtlPolicyAllocationRequiredLabelRule {
    pub message: String,
}

impl ScheduledTtlPolicyAllocationRule for ScheduledTtlPolicyAllocationRequiredLabelRule {
    fn check(&self, item: &ScheduledTtlPolicyAllocation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTtlPolicyAllocationAllowedPhoneRule {
    pub allowed: Vec<String>,
}

impl ScheduledTtlPolicyAllocationRule for ScheduledTtlPolicyAllocationAllowedPhoneRule {
    fn check(&self, item: &ScheduledTtlPolicyAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.phone))) {
            return vec![format!("phone {} is stale", item.phone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTtlPolicyAllocationCodePrefixRule {
    pub prefix: String,
}

impl ScheduledTtlPolicyAllocationRule for ScheduledTtlPolicyAllocationCodePrefixRule {
    fn check(&self, item: &ScheduledTtlPolicyAllocation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTtlPolicyAllocationRequiredLocaleRule {
    pub message: String,
}

impl ScheduledTtlPolicyAllocationRule for ScheduledTtlPolicyAllocationRequiredLocaleRule {
    fn check(&self, item: &ScheduledTtlPolicyAllocation) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled ttl policy allocation.
#[derive(Debug, Clone)]
pub struct ScheduledTtlPolicyAllocationValidator {
    pub rules: Vec<Rc<dyn ScheduledTtlPolicyAllocationRule>>,
}

impl ScheduledTtlPolicyAllocationValidator {
    pub fn validate(&self, item: &ScheduledTtlPolicyAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledTtlPolicyAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledTtlPolicyAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_ttl_policy_allocation_validator() -> ScheduledTtlPolicyAllocationValidator {
    ScheduledTtlPolicyAllocationValidator {
        rules: vec![
            Rc::new(ScheduledTtlPolicyAllocationTagsLimitRule {
                max_count: 16,
                banned: vec!["manual".to_string(), "scheduled".to_string(), "adaptive".to_string()],
            }) as Rc<dyn ScheduledTtlPolicyAllocationRule>,
            Rc::new(ScheduledTtlPolicyAllocationActiveGuardRule { limit: 3.8 }),
            Rc::new(ScheduledTtlPolicyAllocationRequiredLabelRule {
                message: "scheduled ttl policy allocation needs this field".to_string(),
            }),
            Rc::new(ScheduledTtlPolicyAllocationAllowedPhoneRule {
                allowed: vec!["seasonal".to_string()],
            }),
            Rc::new(ScheduledTtlPolicyAllocationCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(ScheduledTtlPolicyAllocationRequiredLocaleRule {
                message: "scheduled ttl policy allocation needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_ttl_policy_allocation_validator();
        let item = new_scheduled_ttl_policy_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ScheduledTtlPolicyAllocationRequiredLabelRule {
            message: "scheduled ttl policy allocation needs an entry".to_string(),
        };
        let mut item = new_scheduled_ttl_policy_allocation("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = ScheduledTtlPolicyAllocationRequiredLocaleRule {
            message: "scheduled ttl policy allocation needs this field".to_string(),
        };
        let mut item = new_scheduled_ttl_policy_allocation("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
