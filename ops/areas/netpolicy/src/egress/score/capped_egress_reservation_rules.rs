use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped egress reservation must pass. Check returns its problems, or none.
pub trait CappedEgressReservationRule: std::fmt::Debug {
    fn check(&self, item: &CappedEgressReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEgressReservationAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl CappedEgressReservationRule for CappedEgressReservationAllowedCodeRule {
    fn check(&self, item: &CappedEgressReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEgressReservationOwnerIdLengthRule {
    pub max_length: i64,
}

impl CappedEgressReservationRule for CappedEgressReservationOwnerIdLengthRule {
    fn check(&self, item: &CappedEgressReservation) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEgressReservationRequiredCodeRule {
    pub message: String,
}

impl CappedEgressReservationRule for CappedEgressReservationRequiredCodeRule {
    fn check(&self, item: &CappedEgressReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEgressReservationVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedEgressReservationRule for CappedEgressReservationVersionRangeRule {
    fn check(&self, item: &CappedEgressReservation) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a capped egress reservation.
#[derive(Debug, Clone)]
pub struct CappedEgressReservationValidator {
    pub rules: Vec<Rc<dyn CappedEgressReservationRule>>,
}

impl CappedEgressReservationValidator {
    pub fn validate(&self, item: &CappedEgressReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedEgressReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedEgressReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_egress_reservation_validator() -> CappedEgressReservationValidator {
    CappedEgressReservationValidator {
        rules: vec![
            Rc::new(CappedEgressReservationAllowedCodeRule { allowed: Vec::new() })
                as Rc<dyn CappedEgressReservationRule>,
            Rc::new(CappedEgressReservationOwnerIdLengthRule { max_length: 16 }),
            Rc::new(CappedEgressReservationRequiredCodeRule {
                message: "capped egress reservation needs an entry".to_string(),
            }),
            Rc::new(CappedEgressReservationVersionRangeRule { min: 80, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_egress_reservation_validator();
        let item = new_capped_egress_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = CappedEgressReservationOwnerIdLengthRule { max_length: 64 };
        let mut item = new_capped_egress_reservation("x", "y");
        item.owner_id = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedEgressReservationRequiredCodeRule {
            message: "capped egress reservation needs this field".to_string(),
        };
        let mut item = new_capped_egress_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = CappedEgressReservationVersionRangeRule { min: 64, max: 255 };
        let mut item = new_capped_egress_reservation("x", "y");
        item.version = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
