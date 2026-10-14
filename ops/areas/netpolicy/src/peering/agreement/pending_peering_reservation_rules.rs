use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending peering reservation must pass. Check returns its problems, or none.
pub trait PendingPeeringReservationRule: std::fmt::Debug {
    fn check(&self, item: &PendingPeeringReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPeeringReservationNameLengthRule {
    pub max_length: i64,
}

impl PendingPeeringReservationRule for PendingPeeringReservationNameLengthRule {
    fn check(&self, item: &PendingPeeringReservation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPeeringReservationTenantIdLengthRule {
    pub max_length: i64,
}

impl PendingPeeringReservationRule for PendingPeeringReservationTenantIdLengthRule {
    fn check(&self, item: &PendingPeeringReservation) -> Vec<String> {
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
pub struct PendingPeeringReservationRequiredCodeRule {
    pub message: String,
}

impl PendingPeeringReservationRule for PendingPeeringReservationRequiredCodeRule {
    fn check(&self, item: &PendingPeeringReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPeeringReservationMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingPeeringReservationRule for PendingPeeringReservationMinimumRangeRule {
    fn check(&self, item: &PendingPeeringReservation) -> Vec<String> {
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
pub struct PendingPeeringReservationRequiredExternalRefRule {
    pub message: String,
}

impl PendingPeeringReservationRule for PendingPeeringReservationRequiredExternalRefRule {
    fn check(&self, item: &PendingPeeringReservation) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending peering reservation.
#[derive(Debug, Clone)]
pub struct PendingPeeringReservationValidator {
    pub rules: Vec<Rc<dyn PendingPeeringReservationRule>>,
}

impl PendingPeeringReservationValidator {
    pub fn validate(&self, item: &PendingPeeringReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingPeeringReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingPeeringReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_peering_reservation_validator() -> PendingPeeringReservationValidator {
    PendingPeeringReservationValidator {
        rules: vec![
            Rc::new(PendingPeeringReservationNameLengthRule { max_length: 120 })
                as Rc<dyn PendingPeeringReservationRule>,
            Rc::new(PendingPeeringReservationTenantIdLengthRule { max_length: 86400 }),
            Rc::new(PendingPeeringReservationRequiredCodeRule {
                message: "pending peering reservation needs a value".to_string(),
            }),
            Rc::new(PendingPeeringReservationMinimumRangeRule { min: 0.1, max: 1922.6 }),
            Rc::new(PendingPeeringReservationRequiredExternalRefRule {
                message: "pending peering reservation needs an entry".to_string(),
            }),
        ],
    }
}
