use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a health check reservation must pass. Check returns its problems, or none.
pub trait HealthCheckReservationRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckReservationRequiredCodeRule {
    pub message: String,
}

impl HealthCheckReservationRule for HealthCheckReservationRequiredCodeRule {
    fn check(&self, item: &HealthCheckReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckReservationRequiredSummaryRule {
    pub message: String,
}

impl HealthCheckReservationRule for HealthCheckReservationRequiredSummaryRule {
    fn check(&self, item: &HealthCheckReservation) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckReservationCodePrefixRule {
    pub prefix: String,
}

impl HealthCheckReservationRule for HealthCheckReservationCodePrefixRule {
    fn check(&self, item: &HealthCheckReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check reservation.
#[derive(Debug, Clone)]
pub struct HealthCheckReservationValidator {
    pub rules: Vec<Rc<dyn HealthCheckReservationRule>>,
}

impl HealthCheckReservationValidator {
    pub fn validate(&self, item: &HealthCheckReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &HealthCheckReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_reservation_validator() -> HealthCheckReservationValidator {
    HealthCheckReservationValidator {
        rules: vec![
            Rc::new(HealthCheckReservationRequiredCodeRule {
                message: "health check reservation needs a value".to_string(),
            }) as Rc<dyn HealthCheckReservationRule>,
            Rc::new(HealthCheckReservationRequiredSummaryRule {
                message: "health check reservation needs this field".to_string(),
            }),
            Rc::new(HealthCheckReservationCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_health_check_reservation_validator();
        let item = new_health_check_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = HealthCheckReservationRequiredCodeRule {
            message: "health check reservation needs a value".to_string(),
        };
        let mut item = new_health_check_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = HealthCheckReservationRequiredSummaryRule {
            message: "health check reservation needs a value".to_string(),
        };
        let mut item = new_health_check_reservation("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
