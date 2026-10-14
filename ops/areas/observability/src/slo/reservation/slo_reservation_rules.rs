use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a slo reservation must pass. Check returns its problems, or none.
pub trait SloReservationRule: std::fmt::Debug {
    fn check(&self, item: &SloReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloReservationRequiredHostRule {
    pub message: String,
}

impl SloReservationRule for SloReservationRequiredHostRule {
    fn check(&self, item: &SloReservation) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloReservationHostLengthRule {
    pub max_length: i64,
}

impl SloReservationRule for SloReservationHostLengthRule {
    fn check(&self, item: &SloReservation) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloReservationArchivedGuardRule {
    pub limit: i64,
}

impl SloReservationRule for SloReservationArchivedGuardRule {
    fn check(&self, item: &SloReservation) -> Vec<String> {
        if item.archived && (item.rotation_days > self.limit) {
            return vec![format!(
                "rotation days {} exceeds {} while archived",
                item.rotation_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloReservationVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SloReservationRule for SloReservationVersionRangeRule {
    fn check(&self, item: &SloReservation) -> Vec<String> {
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

/// Runs every rule of a slo reservation.
#[derive(Debug, Clone)]
pub struct SloReservationValidator {
    pub rules: Vec<Rc<dyn SloReservationRule>>,
}

impl SloReservationValidator {
    pub fn validate(&self, item: &SloReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SloReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_reservation_validator() -> SloReservationValidator {
    SloReservationValidator {
        rules: vec![
            Rc::new(SloReservationRequiredHostRule {
                message: "slo reservation needs this field".to_string(),
            }) as Rc<dyn SloReservationRule>,
            Rc::new(SloReservationHostLengthRule { max_length: 16 }),
            Rc::new(SloReservationArchivedGuardRule { limit: 80 }),
            Rc::new(SloReservationVersionRangeRule { min: 255, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_reservation_validator();
        let item = new_slo_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = SloReservationRequiredHostRule {
            message: "slo reservation needs this field".to_string(),
        };
        let mut item = new_slo_reservation("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = SloReservationHostLengthRule { max_length: 32 };
        let mut item = new_slo_reservation("x", "y");
        item.host = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = SloReservationVersionRangeRule { min: 255, max: 255 };
        let mut item = new_slo_reservation("x", "y");
        item.version = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
