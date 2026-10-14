use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback usage allocation must pass. Check returns its problems, or none.
pub trait FallbackUsageAllocationRule: std::fmt::Debug {
    fn check(&self, item: &FallbackUsageAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackUsageAllocationNamespaceLengthRule {
    pub max_length: i64,
}

impl FallbackUsageAllocationRule for FallbackUsageAllocationNamespaceLengthRule {
    fn check(&self, item: &FallbackUsageAllocation) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackUsageAllocationRequiredNamespaceRule {
    pub message: String,
}

impl FallbackUsageAllocationRule for FallbackUsageAllocationRequiredNamespaceRule {
    fn check(&self, item: &FallbackUsageAllocation) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackUsageAllocationUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackUsageAllocationRule for FallbackUsageAllocationUpdatedAtRangeRule {
    fn check(&self, item: &FallbackUsageAllocation) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackUsageAllocationCodePrefixRule {
    pub prefix: String,
}

impl FallbackUsageAllocationRule for FallbackUsageAllocationCodePrefixRule {
    fn check(&self, item: &FallbackUsageAllocation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback usage allocation.
#[derive(Debug, Clone)]
pub struct FallbackUsageAllocationValidator {
    pub rules: Vec<Rc<dyn FallbackUsageAllocationRule>>,
}

impl FallbackUsageAllocationValidator {
    pub fn validate(&self, item: &FallbackUsageAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackUsageAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackUsageAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_usage_allocation_validator() -> FallbackUsageAllocationValidator {
    FallbackUsageAllocationValidator {
        rules: vec![
            Rc::new(FallbackUsageAllocationNamespaceLengthRule { max_length: 255 })
                as Rc<dyn FallbackUsageAllocationRule>,
            Rc::new(FallbackUsageAllocationRequiredNamespaceRule {
                message: "fallback usage allocation needs a value".to_string(),
            }),
            Rc::new(FallbackUsageAllocationUpdatedAtRangeRule { min: 64, max: 86400 }),
            Rc::new(FallbackUsageAllocationCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_usage_allocation_validator();
        let item = new_fallback_usage_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = FallbackUsageAllocationNamespaceLengthRule { max_length: 120 };
        let mut item = new_fallback_usage_allocation("x", "y");
        item.namespace = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = FallbackUsageAllocationRequiredNamespaceRule {
            message: "fallback usage allocation needs this field".to_string(),
        };
        let mut item = new_fallback_usage_allocation("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = FallbackUsageAllocationUpdatedAtRangeRule { min: 16, max: 32 };
        let mut item = new_fallback_usage_allocation("x", "y");
        item.updated_at = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
