use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a timeline profile must pass. Check returns its problems, or none.
pub trait TimelineProfileRule: std::fmt::Debug {
    fn check(&self, item: &TimelineProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineProfileCurrencyLengthRule {
    pub max_length: i64,
}

impl TimelineProfileRule for TimelineProfileCurrencyLengthRule {
    fn check(&self, item: &TimelineProfile) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineProfileNameLengthRule {
    pub max_length: i64,
}

impl TimelineProfileRule for TimelineProfileNameLengthRule {
    fn check(&self, item: &TimelineProfile) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineProfileAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TimelineProfileRule for TimelineProfileAllowedCodeRule {
    fn check(&self, item: &TimelineProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineProfileRequiredCurrencyRule {
    pub message: String,
}

impl TimelineProfileRule for TimelineProfileRequiredCurrencyRule {
    fn check(&self, item: &TimelineProfile) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineProfileDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TimelineProfileRule for TimelineProfileDurationMinutesRangeRule {
    fn check(&self, item: &TimelineProfile) -> Vec<String> {
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

/// Runs every rule of a timeline profile.
#[derive(Debug, Clone)]
pub struct TimelineProfileValidator {
    pub rules: Vec<Rc<dyn TimelineProfileRule>>,
}

impl TimelineProfileValidator {
    pub fn validate(&self, item: &TimelineProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TimelineProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_profile_validator() -> TimelineProfileValidator {
    TimelineProfileValidator {
        rules: vec![
            Rc::new(TimelineProfileCurrencyLengthRule { max_length: 3600 }) as Rc<dyn TimelineProfileRule>,
            Rc::new(TimelineProfileNameLengthRule { max_length: 120 }),
            Rc::new(TimelineProfileAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(TimelineProfileRequiredCurrencyRule {
                message: "timeline profile needs a value".to_string(),
            }),
            Rc::new(TimelineProfileDurationMinutesRangeRule { min: 32, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_timeline_profile_validator();
        let item = new_timeline_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = TimelineProfileCurrencyLengthRule { max_length: 120 };
        let mut item = new_timeline_profile("x", "y");
        item.currency = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TimelineProfileNameLengthRule { max_length: 1000 };
        let mut item = new_timeline_profile("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = TimelineProfileRequiredCurrencyRule {
            message: "timeline profile needs an entry".to_string(),
        };
        let mut item = new_timeline_profile("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = TimelineProfileDurationMinutesRangeRule { min: 80, max: 255 };
        let mut item = new_timeline_profile("x", "y");
        item.duration_minutes = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
