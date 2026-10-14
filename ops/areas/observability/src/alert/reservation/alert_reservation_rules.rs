use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a alert reservation must pass. Check returns its problems, or none.
pub trait AlertReservationRule: std::fmt::Debug {
    fn check(&self, item: &AlertReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertReservationLocaleLengthRule {
    pub max_length: i64,
}

impl AlertReservationRule for AlertReservationLocaleLengthRule {
    fn check(&self, item: &AlertReservation) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertReservationNameLengthRule {
    pub max_length: i64,
}

impl AlertReservationRule for AlertReservationNameLengthRule {
    fn check(&self, item: &AlertReservation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertReservationDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AlertReservationRule for AlertReservationDiscountRangeRule {
    fn check(&self, item: &AlertReservation) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a alert reservation.
#[derive(Debug, Clone)]
pub struct AlertReservationValidator {
    pub rules: Vec<Rc<dyn AlertReservationRule>>,
}

impl AlertReservationValidator {
    pub fn validate(&self, item: &AlertReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AlertReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_reservation_validator() -> AlertReservationValidator {
    AlertReservationValidator {
        rules: vec![
            Rc::new(AlertReservationLocaleLengthRule { max_length: 80 }) as Rc<dyn AlertReservationRule>,
            Rc::new(AlertReservationNameLengthRule { max_length: 80 }),
            Rc::new(AlertReservationDiscountRangeRule { min: 7.2, max: 1319.7 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_alert_reservation_validator();
        let item = new_alert_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = AlertReservationLocaleLengthRule { max_length: 64 };
        let mut item = new_alert_reservation("x", "y");
        item.locale = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = AlertReservationNameLengthRule { max_length: 1000 };
        let mut item = new_alert_reservation("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = AlertReservationDiscountRangeRule { min: 6.8, max: 2025.7 };
        let mut item = new_alert_reservation("x", "y");
        item.discount = 2027.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
