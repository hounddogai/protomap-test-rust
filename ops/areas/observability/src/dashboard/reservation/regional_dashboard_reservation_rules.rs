use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional dashboard reservation must pass. Check returns its problems, or none.
pub trait RegionalDashboardReservationRule: std::fmt::Debug {
    fn check(&self, item: &RegionalDashboardReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDashboardReservationArchivedGuardRule {
    pub limit: i64,
}

impl RegionalDashboardReservationRule for RegionalDashboardReservationArchivedGuardRule {
    fn check(&self, item: &RegionalDashboardReservation) -> Vec<String> {
        if item.archived && (item.capacity > self.limit) {
            return vec![format!(
                "capacity {} exceeds {} while archived",
                item.capacity, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDashboardReservationCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalDashboardReservationRule for RegionalDashboardReservationCeilingRangeRule {
    fn check(&self, item: &RegionalDashboardReservation) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDashboardReservationRequiredTitleRule {
    pub message: String,
}

impl RegionalDashboardReservationRule for RegionalDashboardReservationRequiredTitleRule {
    fn check(&self, item: &RegionalDashboardReservation) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDashboardReservationTitleLengthRule {
    pub max_length: i64,
}

impl RegionalDashboardReservationRule for RegionalDashboardReservationTitleLengthRule {
    fn check(&self, item: &RegionalDashboardReservation) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDashboardReservationRegionLengthRule {
    pub max_length: i64,
}

impl RegionalDashboardReservationRule for RegionalDashboardReservationRegionLengthRule {
    fn check(&self, item: &RegionalDashboardReservation) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDashboardReservationPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalDashboardReservationRule for RegionalDashboardReservationPortRangeRule {
    fn check(&self, item: &RegionalDashboardReservation) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a regional dashboard reservation.
#[derive(Debug, Clone)]
pub struct RegionalDashboardReservationValidator {
    pub rules: Vec<Rc<dyn RegionalDashboardReservationRule>>,
}

impl RegionalDashboardReservationValidator {
    pub fn validate(&self, item: &RegionalDashboardReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalDashboardReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalDashboardReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_dashboard_reservation_validator() -> RegionalDashboardReservationValidator {
    RegionalDashboardReservationValidator {
        rules: vec![
            Rc::new(RegionalDashboardReservationArchivedGuardRule { limit: 1000 })
                as Rc<dyn RegionalDashboardReservationRule>,
            Rc::new(RegionalDashboardReservationCeilingRangeRule { min: 6.9, max: 1759.5 }),
            Rc::new(RegionalDashboardReservationRequiredTitleRule {
                message: "regional dashboard reservation needs a value".to_string(),
            }),
            Rc::new(RegionalDashboardReservationTitleLengthRule { max_length: 16 }),
            Rc::new(RegionalDashboardReservationRegionLengthRule { max_length: 120 }),
            Rc::new(RegionalDashboardReservationPortRangeRule { min: 255, max: 255 }),
        ],
    }
}
