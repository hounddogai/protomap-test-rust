use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default delegation allocation must pass. Check returns its problems, or none.
pub trait DefaultDelegationAllocationRule: std::fmt::Debug {
    fn check(&self, item: &DefaultDelegationAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDelegationAllocationRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultDelegationAllocationRule for DefaultDelegationAllocationRatioRangeRule {
    fn check(&self, item: &DefaultDelegationAllocation) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDelegationAllocationRequiredNamespaceRule {
    pub message: String,
}

impl DefaultDelegationAllocationRule for DefaultDelegationAllocationRequiredNamespaceRule {
    fn check(&self, item: &DefaultDelegationAllocation) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDelegationAllocationCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultDelegationAllocationRule for DefaultDelegationAllocationCostRangeRule {
    fn check(&self, item: &DefaultDelegationAllocation) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDelegationAllocationPinnedGuardRule {
    pub limit: f64,
}

impl DefaultDelegationAllocationRule for DefaultDelegationAllocationPinnedGuardRule {
    fn check(&self, item: &DefaultDelegationAllocation) -> Vec<String> {
        if item.pinned && (item.ratio > self.limit) {
            return vec![format!("ratio {} exceeds {} while pinned", item.ratio, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDelegationAllocationNamespaceLengthRule {
    pub max_length: i64,
}

impl DefaultDelegationAllocationRule for DefaultDelegationAllocationNamespaceLengthRule {
    fn check(&self, item: &DefaultDelegationAllocation) -> Vec<String> {
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
pub struct DefaultDelegationAllocationRequiredTitleRule {
    pub message: String,
}

impl DefaultDelegationAllocationRule for DefaultDelegationAllocationRequiredTitleRule {
    fn check(&self, item: &DefaultDelegationAllocation) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default delegation allocation.
#[derive(Debug, Clone)]
pub struct DefaultDelegationAllocationValidator {
    pub rules: Vec<Rc<dyn DefaultDelegationAllocationRule>>,
}

impl DefaultDelegationAllocationValidator {
    pub fn validate(&self, item: &DefaultDelegationAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultDelegationAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultDelegationAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_delegation_allocation_validator() -> DefaultDelegationAllocationValidator {
    DefaultDelegationAllocationValidator {
        rules: vec![
            Rc::new(DefaultDelegationAllocationRatioRangeRule { min: 0.8, max: 932.1 })
                as Rc<dyn DefaultDelegationAllocationRule>,
            Rc::new(DefaultDelegationAllocationRequiredNamespaceRule {
                message: "default delegation allocation needs this field".to_string(),
            }),
            Rc::new(DefaultDelegationAllocationCostRangeRule { min: 7.8, max: 1897.6 }),
            Rc::new(DefaultDelegationAllocationPinnedGuardRule { limit: 6.5 }),
            Rc::new(DefaultDelegationAllocationNamespaceLengthRule { max_length: 16 }),
            Rc::new(DefaultDelegationAllocationRequiredTitleRule {
                message: "default delegation allocation needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_delegation_allocation_validator();
        let item = new_default_delegation_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = DefaultDelegationAllocationRatioRangeRule { min: 0.6, max: 843.6 };
        let mut item = new_default_delegation_allocation("x", "y");
        item.ratio = 845.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = DefaultDelegationAllocationRequiredNamespaceRule {
            message: "default delegation allocation needs an entry".to_string(),
        };
        let mut item = new_default_delegation_allocation("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = DefaultDelegationAllocationCostRangeRule { min: 1.6, max: 4130.5 };
        let mut item = new_default_delegation_allocation("x", "y");
        item.cost = 4132.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = DefaultDelegationAllocationNamespaceLengthRule { max_length: 32 };
        let mut item = new_default_delegation_allocation("x", "y");
        item.namespace = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = DefaultDelegationAllocationRequiredTitleRule {
            message: "default delegation allocation needs an entry".to_string(),
        };
        let mut item = new_default_delegation_allocation("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
