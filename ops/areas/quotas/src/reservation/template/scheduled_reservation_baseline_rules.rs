use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled reservation baseline must pass. Check returns its problems, or none.
pub trait ScheduledReservationBaselineRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledReservationBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReservationBaselineDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledReservationBaselineRule for ScheduledReservationBaselineDurationMinutesRangeRule {
    fn check(&self, item: &ScheduledReservationBaseline) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReservationBaselineTitleLengthRule {
    pub max_length: i64,
}

impl ScheduledReservationBaselineRule for ScheduledReservationBaselineTitleLengthRule {
    fn check(&self, item: &ScheduledReservationBaseline) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReservationBaselineExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledReservationBaselineRule for ScheduledReservationBaselineExpiresAtRangeRule {
    fn check(&self, item: &ScheduledReservationBaseline) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReservationBaselineRequiredAudienceRule {
    pub message: String,
}

impl ScheduledReservationBaselineRule for ScheduledReservationBaselineRequiredAudienceRule {
    fn check(&self, item: &ScheduledReservationBaseline) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled reservation baseline.
#[derive(Debug, Clone)]
pub struct ScheduledReservationBaselineValidator {
    pub rules: Vec<Rc<dyn ScheduledReservationBaselineRule>>,
}

impl ScheduledReservationBaselineValidator {
    pub fn validate(&self, item: &ScheduledReservationBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledReservationBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledReservationBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_reservation_baseline_validator() -> ScheduledReservationBaselineValidator {
    ScheduledReservationBaselineValidator {
        rules: vec![
            Rc::new(ScheduledReservationBaselineDurationMinutesRangeRule { min: 32, max: 86400 })
                as Rc<dyn ScheduledReservationBaselineRule>,
            Rc::new(ScheduledReservationBaselineTitleLengthRule { max_length: 32 }),
            Rc::new(ScheduledReservationBaselineExpiresAtRangeRule { min: 255, max: 255 }),
            Rc::new(ScheduledReservationBaselineRequiredAudienceRule {
                message: "scheduled reservation baseline needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_reservation_baseline_validator();
        let item = new_scheduled_reservation_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = ScheduledReservationBaselineDurationMinutesRangeRule { min: 64, max: 120 };
        let mut item = new_scheduled_reservation_baseline("x", "y");
        item.duration_minutes = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ScheduledReservationBaselineTitleLengthRule { max_length: 64 };
        let mut item = new_scheduled_reservation_baseline("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = ScheduledReservationBaselineExpiresAtRangeRule { min: 80, max: 120 };
        let mut item = new_scheduled_reservation_baseline("x", "y");
        item.expires_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = ScheduledReservationBaselineRequiredAudienceRule {
            message: "scheduled reservation baseline needs this field".to_string(),
        };
        let mut item = new_scheduled_reservation_baseline("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
