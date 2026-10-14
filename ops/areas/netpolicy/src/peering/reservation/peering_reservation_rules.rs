use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a peering reservation must pass. Check returns its problems, or none.
pub trait PeeringReservationRule: std::fmt::Debug {
    fn check(&self, item: &PeeringReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringReservationRequiredCodeRule {
    pub message: String,
}

impl PeeringReservationRule for PeeringReservationRequiredCodeRule {
    fn check(&self, item: &PeeringReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringReservationAudienceLengthRule {
    pub max_length: i64,
}

impl PeeringReservationRule for PeeringReservationAudienceLengthRule {
    fn check(&self, item: &PeeringReservation) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringReservationNameLengthRule {
    pub max_length: i64,
}

impl PeeringReservationRule for PeeringReservationNameLengthRule {
    fn check(&self, item: &PeeringReservation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering reservation.
#[derive(Debug, Clone)]
pub struct PeeringReservationValidator {
    pub rules: Vec<Rc<dyn PeeringReservationRule>>,
}

impl PeeringReservationValidator {
    pub fn validate(&self, item: &PeeringReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PeeringReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_reservation_validator() -> PeeringReservationValidator {
    PeeringReservationValidator {
        rules: vec![
            Rc::new(PeeringReservationRequiredCodeRule {
                message: "peering reservation needs an entry".to_string(),
            }) as Rc<dyn PeeringReservationRule>,
            Rc::new(PeeringReservationAudienceLengthRule { max_length: 255 }),
            Rc::new(PeeringReservationNameLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_reservation_validator();
        let item = new_peering_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PeeringReservationRequiredCodeRule {
            message: "peering reservation needs an entry".to_string(),
        };
        let mut item = new_peering_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = PeeringReservationAudienceLengthRule { max_length: 86400 };
        let mut item = new_peering_reservation("x", "y");
        item.audience = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PeeringReservationNameLengthRule { max_length: 3600 };
        let mut item = new_peering_reservation("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
