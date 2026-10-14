use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive dns record reservation must pass. Check returns its problems, or none.
pub trait AdaptiveDnsRecordReservationRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveDnsRecordReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDnsRecordReservationRequiredCodeRule {
    pub message: String,
}

impl AdaptiveDnsRecordReservationRule for AdaptiveDnsRecordReservationRequiredCodeRule {
    fn check(&self, item: &AdaptiveDnsRecordReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDnsRecordReservationActiveGuardRule {
    pub limit: f64,
}

impl AdaptiveDnsRecordReservationRule for AdaptiveDnsRecordReservationActiveGuardRule {
    fn check(&self, item: &AdaptiveDnsRecordReservation) -> Vec<String> {
        if item.active && (item.amount > self.limit) {
            return vec![format!("amount {} exceeds {} while active", item.amount, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDnsRecordReservationAudienceLengthRule {
    pub max_length: i64,
}

impl AdaptiveDnsRecordReservationRule for AdaptiveDnsRecordReservationAudienceLengthRule {
    fn check(&self, item: &AdaptiveDnsRecordReservation) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive dns record reservation.
#[derive(Debug, Clone)]
pub struct AdaptiveDnsRecordReservationValidator {
    pub rules: Vec<Rc<dyn AdaptiveDnsRecordReservationRule>>,
}

impl AdaptiveDnsRecordReservationValidator {
    pub fn validate(&self, item: &AdaptiveDnsRecordReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveDnsRecordReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveDnsRecordReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_dns_record_reservation_validator() -> AdaptiveDnsRecordReservationValidator {
    AdaptiveDnsRecordReservationValidator {
        rules: vec![
            Rc::new(AdaptiveDnsRecordReservationRequiredCodeRule {
                message: "adaptive dns record reservation needs an entry".to_string(),
            }) as Rc<dyn AdaptiveDnsRecordReservationRule>,
            Rc::new(AdaptiveDnsRecordReservationActiveGuardRule { limit: 9.2 }),
            Rc::new(AdaptiveDnsRecordReservationAudienceLengthRule { max_length: 255 }),
        ],
    }
}
