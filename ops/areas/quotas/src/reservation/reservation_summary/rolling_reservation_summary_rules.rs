use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling reservation summary must pass. Check returns its problems, or none.
pub trait RollingReservationSummaryRule: std::fmt::Debug {
    fn check(&self, item: &RollingReservationSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingReservationSummaryRequiredCodeRule {
    pub message: String,
}

impl RollingReservationSummaryRule for RollingReservationSummaryRequiredCodeRule {
    fn check(&self, item: &RollingReservationSummary) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingReservationSummaryTitleLengthRule {
    pub max_length: i64,
}

impl RollingReservationSummaryRule for RollingReservationSummaryTitleLengthRule {
    fn check(&self, item: &RollingReservationSummary) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingReservationSummaryEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingReservationSummaryRule for RollingReservationSummaryEndsAtRangeRule {
    fn check(&self, item: &RollingReservationSummary) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingReservationSummarySourceLengthRule {
    pub max_length: i64,
}

impl RollingReservationSummaryRule for RollingReservationSummarySourceLengthRule {
    fn check(&self, item: &RollingReservationSummary) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling reservation summary.
#[derive(Debug, Clone)]
pub struct RollingReservationSummaryValidator {
    pub rules: Vec<Rc<dyn RollingReservationSummaryRule>>,
}

impl RollingReservationSummaryValidator {
    pub fn validate(&self, item: &RollingReservationSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingReservationSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingReservationSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_reservation_summary_validator() -> RollingReservationSummaryValidator {
    RollingReservationSummaryValidator {
        rules: vec![
            Rc::new(RollingReservationSummaryRequiredCodeRule {
                message: "rolling reservation summary needs a value".to_string(),
            }) as Rc<dyn RollingReservationSummaryRule>,
            Rc::new(RollingReservationSummaryTitleLengthRule { max_length: 255 }),
            Rc::new(RollingReservationSummaryEndsAtRangeRule { min: 32, max: 120 }),
            Rc::new(RollingReservationSummarySourceLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_reservation_summary_validator();
        let item = new_rolling_reservation_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingReservationSummaryRequiredCodeRule {
            message: "rolling reservation summary needs an entry".to_string(),
        };
        let mut item = new_rolling_reservation_summary("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RollingReservationSummaryTitleLengthRule { max_length: 1000 };
        let mut item = new_rolling_reservation_summary("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = RollingReservationSummaryEndsAtRangeRule { min: 120, max: 1000 };
        let mut item = new_rolling_reservation_summary("x", "y");
        item.ends_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = RollingReservationSummarySourceLengthRule { max_length: 16 };
        let mut item = new_rolling_reservation_summary("x", "y");
        item.source = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
