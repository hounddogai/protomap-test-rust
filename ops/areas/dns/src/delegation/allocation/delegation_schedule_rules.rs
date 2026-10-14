use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a delegation schedule must pass. Check returns its problems, or none.
pub trait DelegationScheduleRule: std::fmt::Debug {
    fn check(&self, item: &DelegationSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationScheduleCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationScheduleRule for DelegationScheduleCapacityRangeRule {
    fn check(&self, item: &DelegationSchedule) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationScheduleTenantIdLengthRule {
    pub max_length: i64,
}

impl DelegationScheduleRule for DelegationScheduleTenantIdLengthRule {
    fn check(&self, item: &DelegationSchedule) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationScheduleCountryLengthRule {
    pub max_length: i64,
}

impl DelegationScheduleRule for DelegationScheduleCountryLengthRule {
    fn check(&self, item: &DelegationSchedule) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation schedule.
#[derive(Debug, Clone)]
pub struct DelegationScheduleValidator {
    pub rules: Vec<Rc<dyn DelegationScheduleRule>>,
}

impl DelegationScheduleValidator {
    pub fn validate(&self, item: &DelegationSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DelegationSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_schedule_validator() -> DelegationScheduleValidator {
    DelegationScheduleValidator {
        rules: vec![
            Rc::new(DelegationScheduleCapacityRangeRule { min: 80, max: 1000 }) as Rc<dyn DelegationScheduleRule>,
            Rc::new(DelegationScheduleTenantIdLengthRule { max_length: 1000 }),
            Rc::new(DelegationScheduleCountryLengthRule { max_length: 16 }),
        ],
    }
}
