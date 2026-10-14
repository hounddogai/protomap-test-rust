use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive reservation profile must pass. Check returns its problems, or none.
pub trait AdaptiveReservationProfileRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveReservationProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveReservationProfileAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl AdaptiveReservationProfileRule for AdaptiveReservationProfileAllowedLabelRule {
    fn check(&self, item: &AdaptiveReservationProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is below minimum", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveReservationProfileLabelLengthRule {
    pub max_length: i64,
}

impl AdaptiveReservationProfileRule for AdaptiveReservationProfileLabelLengthRule {
    fn check(&self, item: &AdaptiveReservationProfile) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveReservationProfileRequiredLabelRule {
    pub message: String,
}

impl AdaptiveReservationProfileRule for AdaptiveReservationProfileRequiredLabelRule {
    fn check(&self, item: &AdaptiveReservationProfile) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveReservationProfileChannelLengthRule {
    pub max_length: i64,
}

impl AdaptiveReservationProfileRule for AdaptiveReservationProfileChannelLengthRule {
    fn check(&self, item: &AdaptiveReservationProfile) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveReservationProfileDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveReservationProfileRule for AdaptiveReservationProfileDurationMinutesRangeRule {
    fn check(&self, item: &AdaptiveReservationProfile) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a adaptive reservation profile.
#[derive(Debug, Clone)]
pub struct AdaptiveReservationProfileValidator {
    pub rules: Vec<Rc<dyn AdaptiveReservationProfileRule>>,
}

impl AdaptiveReservationProfileValidator {
    pub fn validate(&self, item: &AdaptiveReservationProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveReservationProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveReservationProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_reservation_profile_validator() -> AdaptiveReservationProfileValidator {
    AdaptiveReservationProfileValidator {
        rules: vec![
            Rc::new(AdaptiveReservationProfileAllowedLabelRule { allowed: Vec::new() })
                as Rc<dyn AdaptiveReservationProfileRule>,
            Rc::new(AdaptiveReservationProfileLabelLengthRule { max_length: 3600 }),
            Rc::new(AdaptiveReservationProfileRequiredLabelRule {
                message: "adaptive reservation profile needs this field".to_string(),
            }),
            Rc::new(AdaptiveReservationProfileChannelLengthRule { max_length: 3600 }),
            Rc::new(AdaptiveReservationProfileDurationMinutesRangeRule { min: 255, max: 86400 }),
        ],
    }
}
