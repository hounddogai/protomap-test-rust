use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a reservation audit must pass. Check returns its problems, or none.
pub trait ReservationAuditRule: std::fmt::Debug {
    fn check(&self, item: &ReservationAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAuditRequiredClusterRule {
    pub message: String,
}

impl ReservationAuditRule for ReservationAuditRequiredClusterRule {
    fn check(&self, item: &ReservationAudit) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAuditCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ReservationAuditRule for ReservationAuditCostRangeRule {
    fn check(&self, item: &ReservationAudit) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAuditTitleLengthRule {
    pub max_length: i64,
}

impl ReservationAuditRule for ReservationAuditTitleLengthRule {
    fn check(&self, item: &ReservationAudit) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation audit.
#[derive(Debug, Clone)]
pub struct ReservationAuditValidator {
    pub rules: Vec<Rc<dyn ReservationAuditRule>>,
}

impl ReservationAuditValidator {
    pub fn validate(&self, item: &ReservationAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReservationAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_audit_validator() -> ReservationAuditValidator {
    ReservationAuditValidator {
        rules: vec![
            Rc::new(ReservationAuditRequiredClusterRule {
                message: "reservation audit needs this field".to_string(),
            }) as Rc<dyn ReservationAuditRule>,
            Rc::new(ReservationAuditCostRangeRule { min: 2.2, max: 4930.1 }),
            Rc::new(ReservationAuditTitleLengthRule { max_length: 64 }),
        ],
    }
}
