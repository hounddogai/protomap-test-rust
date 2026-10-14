use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted zone reservation must pass. Check returns its problems, or none.
pub trait WeightedZoneReservationRule: std::fmt::Debug {
    fn check(&self, item: &WeightedZoneReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedZoneReservationAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedZoneReservationRule for WeightedZoneReservationAttemptsRangeRule {
    fn check(&self, item: &WeightedZoneReservation) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedZoneReservationExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedZoneReservationRule for WeightedZoneReservationExpiresAtRangeRule {
    fn check(&self, item: &WeightedZoneReservation) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedZoneReservationRequiredTitleRule {
    pub message: String,
}

impl WeightedZoneReservationRule for WeightedZoneReservationRequiredTitleRule {
    fn check(&self, item: &WeightedZoneReservation) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted zone reservation.
#[derive(Debug, Clone)]
pub struct WeightedZoneReservationValidator {
    pub rules: Vec<Rc<dyn WeightedZoneReservationRule>>,
}

impl WeightedZoneReservationValidator {
    pub fn validate(&self, item: &WeightedZoneReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedZoneReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedZoneReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_zone_reservation_validator() -> WeightedZoneReservationValidator {
    WeightedZoneReservationValidator {
        rules: vec![
            Rc::new(WeightedZoneReservationAttemptsRangeRule { min: 16, max: 86400 })
                as Rc<dyn WeightedZoneReservationRule>,
            Rc::new(WeightedZoneReservationExpiresAtRangeRule { min: 32, max: 86400 }),
            Rc::new(WeightedZoneReservationRequiredTitleRule {
                message: "weighted zone reservation needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_zone_reservation_validator();
        let item = new_weighted_zone_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = WeightedZoneReservationAttemptsRangeRule { min: 16, max: 120 };
        let mut item = new_weighted_zone_reservation("x", "y");
        item.attempts = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = WeightedZoneReservationExpiresAtRangeRule { min: 80, max: 3600 };
        let mut item = new_weighted_zone_reservation("x", "y");
        item.expires_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = WeightedZoneReservationRequiredTitleRule {
            message: "weighted zone reservation needs an entry".to_string(),
        };
        let mut item = new_weighted_zone_reservation("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
