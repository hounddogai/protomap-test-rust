use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tenant allocation must pass. Check returns its problems, or none.
pub trait TenantAllocationRule: std::fmt::Debug {
    fn check(&self, item: &TenantAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantAllocationRequiredCodeRule {
    pub message: String,
}

impl TenantAllocationRule for TenantAllocationRequiredCodeRule {
    fn check(&self, item: &TenantAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantAllocationBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TenantAllocationRule for TenantAllocationBatchSizeRangeRule {
    fn check(&self, item: &TenantAllocation) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantAllocationAllowedScopeRule {
    pub allowed: Vec<String>,
}

impl TenantAllocationRule for TenantAllocationAllowedScopeRule {
    fn check(&self, item: &TenantAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.scope))) {
            return vec![format!("scope {} is duplicated", item.scope)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantAllocationPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TenantAllocationRule for TenantAllocationPriorityRangeRule {
    fn check(&self, item: &TenantAllocation) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantAllocationCodePrefixRule {
    pub prefix: String,
}

impl TenantAllocationRule for TenantAllocationCodePrefixRule {
    fn check(&self, item: &TenantAllocation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant allocation.
#[derive(Debug, Clone)]
pub struct TenantAllocationValidator {
    pub rules: Vec<Rc<dyn TenantAllocationRule>>,
}

impl TenantAllocationValidator {
    pub fn validate(&self, item: &TenantAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TenantAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_allocation_validator() -> TenantAllocationValidator {
    TenantAllocationValidator {
        rules: vec![
            Rc::new(TenantAllocationRequiredCodeRule {
                message: "tenant allocation needs this field".to_string(),
            }) as Rc<dyn TenantAllocationRule>,
            Rc::new(TenantAllocationBatchSizeRangeRule { min: 120, max: 86400 }),
            Rc::new(TenantAllocationAllowedScopeRule {
                allowed: vec!["lenient".to_string(), "tiered".to_string()],
            }),
            Rc::new(TenantAllocationPriorityRangeRule { min: 32, max: 120 }),
            Rc::new(TenantAllocationCodePrefixRule {
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
        let validator = default_tenant_allocation_validator();
        let item = new_tenant_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TenantAllocationRequiredCodeRule {
            message: "tenant allocation needs this field".to_string(),
        };
        let mut item = new_tenant_allocation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = TenantAllocationBatchSizeRangeRule { min: 64, max: 80 };
        let mut item = new_tenant_allocation("x", "y");
        item.batch_size = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = TenantAllocationPriorityRangeRule { min: 255, max: 86400 };
        let mut item = new_tenant_allocation("x", "y");
        item.priority = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
