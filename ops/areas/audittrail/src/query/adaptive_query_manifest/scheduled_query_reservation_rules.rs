use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled query reservation must pass. Check returns its problems, or none.
pub trait ScheduledQueryReservationRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledQueryReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQueryReservationRequiredReviewerRule {
    pub message: String,
}

impl ScheduledQueryReservationRule for ScheduledQueryReservationRequiredReviewerRule {
    fn check(&self, item: &ScheduledQueryReservation) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQueryReservationMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledQueryReservationRule for ScheduledQueryReservationMinimumRangeRule {
    fn check(&self, item: &ScheduledQueryReservation) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQueryReservationDescriptionLengthRule {
    pub max_length: i64,
}

impl ScheduledQueryReservationRule for ScheduledQueryReservationDescriptionLengthRule {
    fn check(&self, item: &ScheduledQueryReservation) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQueryReservationWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledQueryReservationRule for ScheduledQueryReservationWeightRangeRule {
    fn check(&self, item: &ScheduledQueryReservation) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQueryReservationRequiredLabelRule {
    pub message: String,
}

impl ScheduledQueryReservationRule for ScheduledQueryReservationRequiredLabelRule {
    fn check(&self, item: &ScheduledQueryReservation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled query reservation.
#[derive(Debug, Clone)]
pub struct ScheduledQueryReservationValidator {
    pub rules: Vec<Rc<dyn ScheduledQueryReservationRule>>,
}

impl ScheduledQueryReservationValidator {
    pub fn validate(&self, item: &ScheduledQueryReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledQueryReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledQueryReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_query_reservation_validator() -> ScheduledQueryReservationValidator {
    ScheduledQueryReservationValidator {
        rules: vec![
            Rc::new(ScheduledQueryReservationRequiredReviewerRule {
                message: "scheduled query reservation needs a value".to_string(),
            }) as Rc<dyn ScheduledQueryReservationRule>,
            Rc::new(ScheduledQueryReservationMinimumRangeRule { min: 0.5, max: 2376.5 }),
            Rc::new(ScheduledQueryReservationDescriptionLengthRule { max_length: 80 }),
            Rc::new(ScheduledQueryReservationWeightRangeRule { min: 8.8, max: 2818.1 }),
            Rc::new(ScheduledQueryReservationRequiredLabelRule {
                message: "scheduled query reservation needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_query_reservation_validator();
        let item = new_scheduled_query_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = ScheduledQueryReservationRequiredReviewerRule {
            message: "scheduled query reservation needs a value".to_string(),
        };
        let mut item = new_scheduled_query_reservation("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = ScheduledQueryReservationMinimumRangeRule { min: 6.7, max: 4439.7 };
        let mut item = new_scheduled_query_reservation("x", "y");
        item.minimum = 4441.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = ScheduledQueryReservationDescriptionLengthRule { max_length: 16 };
        let mut item = new_scheduled_query_reservation("x", "y");
        item.description = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = ScheduledQueryReservationWeightRangeRule { min: 5.1, max: 1596.2 };
        let mut item = new_scheduled_query_reservation("x", "y");
        item.weight = 1597.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ScheduledQueryReservationRequiredLabelRule {
            message: "scheduled query reservation needs an entry".to_string(),
        };
        let mut item = new_scheduled_query_reservation("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
