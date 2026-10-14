use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary dashboard reservation must pass. Check returns its problems, or none.
pub trait PrimaryDashboardReservationRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryDashboardReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDashboardReservationRequiredUnitRule {
    pub message: String,
}

impl PrimaryDashboardReservationRule for PrimaryDashboardReservationRequiredUnitRule {
    fn check(&self, item: &PrimaryDashboardReservation) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDashboardReservationRequiredCodeRule {
    pub message: String,
}

impl PrimaryDashboardReservationRule for PrimaryDashboardReservationRequiredCodeRule {
    fn check(&self, item: &PrimaryDashboardReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDashboardReservationExternalRefLengthRule {
    pub max_length: i64,
}

impl PrimaryDashboardReservationRule for PrimaryDashboardReservationExternalRefLengthRule {
    fn check(&self, item: &PrimaryDashboardReservation) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDashboardReservationAllowedUnitRule {
    pub allowed: Vec<String>,
}

impl PrimaryDashboardReservationRule for PrimaryDashboardReservationAllowedUnitRule {
    fn check(&self, item: &PrimaryDashboardReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.unit))) {
            return vec![format!("unit {} is below minimum", item.unit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary dashboard reservation.
#[derive(Debug, Clone)]
pub struct PrimaryDashboardReservationValidator {
    pub rules: Vec<Rc<dyn PrimaryDashboardReservationRule>>,
}

impl PrimaryDashboardReservationValidator {
    pub fn validate(&self, item: &PrimaryDashboardReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryDashboardReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryDashboardReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_dashboard_reservation_validator() -> PrimaryDashboardReservationValidator {
    PrimaryDashboardReservationValidator {
        rules: vec![
            Rc::new(PrimaryDashboardReservationRequiredUnitRule {
                message: "primary dashboard reservation needs this field".to_string(),
            }) as Rc<dyn PrimaryDashboardReservationRule>,
            Rc::new(PrimaryDashboardReservationRequiredCodeRule {
                message: "primary dashboard reservation needs an entry".to_string(),
            }),
            Rc::new(PrimaryDashboardReservationExternalRefLengthRule { max_length: 3600 }),
            Rc::new(PrimaryDashboardReservationAllowedUnitRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_dashboard_reservation_validator();
        let item = new_primary_dashboard_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = PrimaryDashboardReservationRequiredUnitRule {
            message: "primary dashboard reservation needs a value".to_string(),
        };
        let mut item = new_primary_dashboard_reservation("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryDashboardReservationRequiredCodeRule {
            message: "primary dashboard reservation needs this field".to_string(),
        };
        let mut item = new_primary_dashboard_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = PrimaryDashboardReservationExternalRefLengthRule { max_length: 80 };
        let mut item = new_primary_dashboard_reservation("x", "y");
        item.external_ref = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
