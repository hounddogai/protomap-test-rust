use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending suspension allocation must pass. Check returns its problems, or none.
pub trait PendingSuspensionAllocationRule: std::fmt::Debug {
    fn check(&self, item: &PendingSuspensionAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSuspensionAllocationRequiredTenantIdRule {
    pub message: String,
}

impl PendingSuspensionAllocationRule for PendingSuspensionAllocationRequiredTenantIdRule {
    fn check(&self, item: &PendingSuspensionAllocation) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSuspensionAllocationQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingSuspensionAllocationRule for PendingSuspensionAllocationQuotaUnitsRangeRule {
    fn check(&self, item: &PendingSuspensionAllocation) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSuspensionAllocationNamespaceLengthRule {
    pub max_length: i64,
}

impl PendingSuspensionAllocationRule for PendingSuspensionAllocationNamespaceLengthRule {
    fn check(&self, item: &PendingSuspensionAllocation) -> Vec<String> {
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

/// Runs every rule of a pending suspension allocation.
#[derive(Debug, Clone)]
pub struct PendingSuspensionAllocationValidator {
    pub rules: Vec<Rc<dyn PendingSuspensionAllocationRule>>,
}

impl PendingSuspensionAllocationValidator {
    pub fn validate(&self, item: &PendingSuspensionAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingSuspensionAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingSuspensionAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_suspension_allocation_validator() -> PendingSuspensionAllocationValidator {
    PendingSuspensionAllocationValidator {
        rules: vec![
            Rc::new(PendingSuspensionAllocationRequiredTenantIdRule {
                message: "pending suspension allocation needs an entry".to_string(),
            }) as Rc<dyn PendingSuspensionAllocationRule>,
            Rc::new(PendingSuspensionAllocationQuotaUnitsRangeRule { min: 64, max: 80 }),
            Rc::new(PendingSuspensionAllocationNamespaceLengthRule { max_length: 3600 }),
        ],
    }
}
