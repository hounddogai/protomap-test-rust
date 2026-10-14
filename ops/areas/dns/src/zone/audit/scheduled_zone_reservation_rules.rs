use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled zone reservation must pass. Check returns its problems, or none.
pub trait ScheduledZoneReservationRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledZoneReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneReservationRequiredCodeRule {
    pub message: String,
}

impl ScheduledZoneReservationRule for ScheduledZoneReservationRequiredCodeRule {
    fn check(&self, item: &ScheduledZoneReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneReservationRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledZoneReservationRule for ScheduledZoneReservationRateRangeRule {
    fn check(&self, item: &ScheduledZoneReservation) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneReservationGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledZoneReservationRule for ScheduledZoneReservationGraceDaysRangeRule {
    fn check(&self, item: &ScheduledZoneReservation) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneReservationRequiredCategoryRule {
    pub message: String,
}

impl ScheduledZoneReservationRule for ScheduledZoneReservationRequiredCategoryRule {
    fn check(&self, item: &ScheduledZoneReservation) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneReservationRequiredLabelRule {
    pub message: String,
}

impl ScheduledZoneReservationRule for ScheduledZoneReservationRequiredLabelRule {
    fn check(&self, item: &ScheduledZoneReservation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled zone reservation.
#[derive(Debug, Clone)]
pub struct ScheduledZoneReservationValidator {
    pub rules: Vec<Rc<dyn ScheduledZoneReservationRule>>,
}

impl ScheduledZoneReservationValidator {
    pub fn validate(&self, item: &ScheduledZoneReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledZoneReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledZoneReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_zone_reservation_validator() -> ScheduledZoneReservationValidator {
    ScheduledZoneReservationValidator {
        rules: vec![
            Rc::new(ScheduledZoneReservationRequiredCodeRule {
                message: "scheduled zone reservation needs this field".to_string(),
            }) as Rc<dyn ScheduledZoneReservationRule>,
            Rc::new(ScheduledZoneReservationRateRangeRule { min: 2.9, max: 4138.3 }),
            Rc::new(ScheduledZoneReservationGraceDaysRangeRule { min: 32, max: 1000 }),
            Rc::new(ScheduledZoneReservationRequiredCategoryRule {
                message: "scheduled zone reservation needs an entry".to_string(),
            }),
            Rc::new(ScheduledZoneReservationRequiredLabelRule {
                message: "scheduled zone reservation needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_zone_reservation_validator();
        let item = new_scheduled_zone_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledZoneReservationRequiredCodeRule {
            message: "scheduled zone reservation needs this field".to_string(),
        };
        let mut item = new_scheduled_zone_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = ScheduledZoneReservationRateRangeRule { min: 2.5, max: 1302.4 };
        let mut item = new_scheduled_zone_reservation("x", "y");
        item.rate = 1303.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = ScheduledZoneReservationGraceDaysRangeRule { min: 16, max: 3600 };
        let mut item = new_scheduled_zone_reservation("x", "y");
        item.grace_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = ScheduledZoneReservationRequiredCategoryRule {
            message: "scheduled zone reservation needs a value".to_string(),
        };
        let mut item = new_scheduled_zone_reservation("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ScheduledZoneReservationRequiredLabelRule {
            message: "scheduled zone reservation needs an entry".to_string(),
        };
        let mut item = new_scheduled_zone_reservation("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
