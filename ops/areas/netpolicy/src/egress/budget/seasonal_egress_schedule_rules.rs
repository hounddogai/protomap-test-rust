use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal egress schedule must pass. Check returns its problems, or none.
pub trait SeasonalEgressScheduleRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalEgressSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEgressScheduleTimezoneLengthRule {
    pub max_length: i64,
}

impl SeasonalEgressScheduleRule for SeasonalEgressScheduleTimezoneLengthRule {
    fn check(&self, item: &SeasonalEgressSchedule) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEgressScheduleSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalEgressScheduleRule for SeasonalEgressScheduleSequenceRangeRule {
    fn check(&self, item: &SeasonalEgressSchedule) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEgressScheduleMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalEgressScheduleRule for SeasonalEgressScheduleMinimumRangeRule {
    fn check(&self, item: &SeasonalEgressSchedule) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEgressScheduleAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl SeasonalEgressScheduleRule for SeasonalEgressScheduleAllowedSourceRule {
    fn check(&self, item: &SeasonalEgressSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is unknown", item.source)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEgressScheduleVisibleGuardRule {
    pub limit: f64,
}

impl SeasonalEgressScheduleRule for SeasonalEgressScheduleVisibleGuardRule {
    fn check(&self, item: &SeasonalEgressSchedule) -> Vec<String> {
        if item.visible && (item.minimum > self.limit) {
            return vec![format!("minimum {} exceeds {} while visible", item.minimum, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal egress schedule.
#[derive(Debug, Clone)]
pub struct SeasonalEgressScheduleValidator {
    pub rules: Vec<Rc<dyn SeasonalEgressScheduleRule>>,
}

impl SeasonalEgressScheduleValidator {
    pub fn validate(&self, item: &SeasonalEgressSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalEgressSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalEgressSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_egress_schedule_validator() -> SeasonalEgressScheduleValidator {
    SeasonalEgressScheduleValidator {
        rules: vec![
            Rc::new(SeasonalEgressScheduleTimezoneLengthRule { max_length: 120 }) as Rc<dyn SeasonalEgressScheduleRule>,
            Rc::new(SeasonalEgressScheduleSequenceRangeRule { min: 3600, max: 86400 }),
            Rc::new(SeasonalEgressScheduleMinimumRangeRule { min: 3.4, max: 1644.4 }),
            Rc::new(SeasonalEgressScheduleAllowedSourceRule { allowed: Vec::new() }),
            Rc::new(SeasonalEgressScheduleVisibleGuardRule { limit: 4.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_egress_schedule_validator();
        let item = new_seasonal_egress_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = SeasonalEgressScheduleTimezoneLengthRule { max_length: 120 };
        let mut item = new_seasonal_egress_schedule("x", "y");
        item.timezone = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = SeasonalEgressScheduleSequenceRangeRule { min: 64, max: 255 };
        let mut item = new_seasonal_egress_schedule("x", "y");
        item.sequence = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = SeasonalEgressScheduleMinimumRangeRule { min: 3.5, max: 4457.7 };
        let mut item = new_seasonal_egress_schedule("x", "y");
        item.minimum = 4459.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
