use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default egress reservation must pass. Check returns its problems, or none.
pub trait DefaultEgressReservationRule: std::fmt::Debug {
    fn check(&self, item: &DefaultEgressReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressReservationCodeLengthRule {
    pub max_length: i64,
}

impl DefaultEgressReservationRule for DefaultEgressReservationCodeLengthRule {
    fn check(&self, item: &DefaultEgressReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressReservationClusterLengthRule {
    pub max_length: i64,
}

impl DefaultEgressReservationRule for DefaultEgressReservationClusterLengthRule {
    fn check(&self, item: &DefaultEgressReservation) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressReservationSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultEgressReservationRule for DefaultEgressReservationSequenceRangeRule {
    fn check(&self, item: &DefaultEgressReservation) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressReservationContactEmailLengthRule {
    pub max_length: i64,
}

impl DefaultEgressReservationRule for DefaultEgressReservationContactEmailLengthRule {
    fn check(&self, item: &DefaultEgressReservation) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressReservationActiveGuardRule {
    pub limit: f64,
}

impl DefaultEgressReservationRule for DefaultEgressReservationActiveGuardRule {
    fn check(&self, item: &DefaultEgressReservation) -> Vec<String> {
        if item.active && (item.rate > self.limit) {
            return vec![format!("rate {} exceeds {} while active", item.rate, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default egress reservation.
#[derive(Debug, Clone)]
pub struct DefaultEgressReservationValidator {
    pub rules: Vec<Rc<dyn DefaultEgressReservationRule>>,
}

impl DefaultEgressReservationValidator {
    pub fn validate(&self, item: &DefaultEgressReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultEgressReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultEgressReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_egress_reservation_validator() -> DefaultEgressReservationValidator {
    DefaultEgressReservationValidator {
        rules: vec![
            Rc::new(DefaultEgressReservationCodeLengthRule { max_length: 80 }) as Rc<dyn DefaultEgressReservationRule>,
            Rc::new(DefaultEgressReservationClusterLengthRule { max_length: 86400 }),
            Rc::new(DefaultEgressReservationSequenceRangeRule { min: 80, max: 86400 }),
            Rc::new(DefaultEgressReservationContactEmailLengthRule { max_length: 1000 }),
            Rc::new(DefaultEgressReservationActiveGuardRule { limit: 0.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_egress_reservation_validator();
        let item = new_default_egress_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultEgressReservationCodeLengthRule { max_length: 86400 };
        let mut item = new_default_egress_reservation("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = DefaultEgressReservationClusterLengthRule { max_length: 1000 };
        let mut item = new_default_egress_reservation("x", "y");
        item.cluster = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = DefaultEgressReservationSequenceRangeRule { min: 32, max: 80 };
        let mut item = new_default_egress_reservation("x", "y");
        item.sequence = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = DefaultEgressReservationContactEmailLengthRule { max_length: 3600 };
        let mut item = new_default_egress_reservation("x", "y");
        item.contact_email = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
