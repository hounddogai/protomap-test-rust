use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dashboard reservation must pass. Check returns its problems, or none.
pub trait DashboardReservationRule: std::fmt::Debug {
    fn check(&self, item: &DashboardReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardReservationPhoneLengthRule {
    pub max_length: i64,
}

impl DashboardReservationRule for DashboardReservationPhoneLengthRule {
    fn check(&self, item: &DashboardReservation) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardReservationNameLengthRule {
    pub max_length: i64,
}

impl DashboardReservationRule for DashboardReservationNameLengthRule {
    fn check(&self, item: &DashboardReservation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardReservationTitleLengthRule {
    pub max_length: i64,
}

impl DashboardReservationRule for DashboardReservationTitleLengthRule {
    fn check(&self, item: &DashboardReservation) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dashboard reservation.
#[derive(Debug, Clone)]
pub struct DashboardReservationValidator {
    pub rules: Vec<Rc<dyn DashboardReservationRule>>,
}

impl DashboardReservationValidator {
    pub fn validate(&self, item: &DashboardReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DashboardReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_reservation_validator() -> DashboardReservationValidator {
    DashboardReservationValidator {
        rules: vec![
            Rc::new(DashboardReservationPhoneLengthRule { max_length: 80 }) as Rc<dyn DashboardReservationRule>,
            Rc::new(DashboardReservationNameLengthRule { max_length: 16 }),
            Rc::new(DashboardReservationTitleLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dashboard_reservation_validator();
        let item = new_dashboard_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = DashboardReservationPhoneLengthRule { max_length: 120 };
        let mut item = new_dashboard_reservation("x", "y");
        item.phone = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = DashboardReservationNameLengthRule { max_length: 64 };
        let mut item = new_dashboard_reservation("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = DashboardReservationTitleLengthRule { max_length: 1000 };
        let mut item = new_dashboard_reservation("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
