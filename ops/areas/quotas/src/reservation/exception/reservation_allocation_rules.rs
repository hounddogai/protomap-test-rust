use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a reservation allocation must pass. Check returns its problems, or none.
pub trait ReservationAllocationRule: std::fmt::Debug {
    fn check(&self, item: &ReservationAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAllocationRequiredLabelRule {
    pub message: String,
}

impl ReservationAllocationRule for ReservationAllocationRequiredLabelRule {
    fn check(&self, item: &ReservationAllocation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAllocationLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReservationAllocationRule for ReservationAllocationLimitRangeRule {
    fn check(&self, item: &ReservationAllocation) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAllocationCodeLengthRule {
    pub max_length: i64,
}

impl ReservationAllocationRule for ReservationAllocationCodeLengthRule {
    fn check(&self, item: &ReservationAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAllocationHostLengthRule {
    pub max_length: i64,
}

impl ReservationAllocationRule for ReservationAllocationHostLengthRule {
    fn check(&self, item: &ReservationAllocation) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAllocationRequiredHostRule {
    pub message: String,
}

impl ReservationAllocationRule for ReservationAllocationRequiredHostRule {
    fn check(&self, item: &ReservationAllocation) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAllocationAllowedTenantIdRule {
    pub allowed: Vec<String>,
}

impl ReservationAllocationRule for ReservationAllocationAllowedTenantIdRule {
    fn check(&self, item: &ReservationAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.tenant_id)))
        {
            return vec![format!("tenant id {} is above maximum", item.tenant_id)];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation allocation.
#[derive(Debug, Clone)]
pub struct ReservationAllocationValidator {
    pub rules: Vec<Rc<dyn ReservationAllocationRule>>,
}

impl ReservationAllocationValidator {
    pub fn validate(&self, item: &ReservationAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReservationAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_allocation_validator() -> ReservationAllocationValidator {
    ReservationAllocationValidator {
        rules: vec![
            Rc::new(ReservationAllocationRequiredLabelRule {
                message: "reservation allocation needs an entry".to_string(),
            }) as Rc<dyn ReservationAllocationRule>,
            Rc::new(ReservationAllocationLimitRangeRule { min: 64, max: 255 }),
            Rc::new(ReservationAllocationCodeLengthRule { max_length: 86400 }),
            Rc::new(ReservationAllocationHostLengthRule { max_length: 80 }),
            Rc::new(ReservationAllocationRequiredHostRule {
                message: "reservation allocation needs this field".to_string(),
            }),
            Rc::new(ReservationAllocationAllowedTenantIdRule { allowed: Vec::new() }),
        ],
    }
}
