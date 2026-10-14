use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a reservation summary must pass. Check returns its problems, or none.
pub trait ReservationSummaryRule: std::fmt::Debug {
    fn check(&self, item: &ReservationSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationSummaryEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReservationSummaryRule for ReservationSummaryEndsAtRangeRule {
    fn check(&self, item: &ReservationSummary) -> Vec<String> {
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
pub struct ReservationSummaryAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl ReservationSummaryRule for ReservationSummaryAllowedAudienceRule {
    fn check(&self, item: &ReservationSummary) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is out of range", item.audience)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationSummaryVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReservationSummaryRule for ReservationSummaryVersionRangeRule {
    fn check(&self, item: &ReservationSummary) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationSummaryLabelLengthRule {
    pub max_length: i64,
}

impl ReservationSummaryRule for ReservationSummaryLabelLengthRule {
    fn check(&self, item: &ReservationSummary) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationSummaryRequiredLabelRule {
    pub message: String,
}

impl ReservationSummaryRule for ReservationSummaryRequiredLabelRule {
    fn check(&self, item: &ReservationSummary) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationSummaryRequiredAudienceRule {
    pub message: String,
}

impl ReservationSummaryRule for ReservationSummaryRequiredAudienceRule {
    fn check(&self, item: &ReservationSummary) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation summary.
#[derive(Debug, Clone)]
pub struct ReservationSummaryValidator {
    pub rules: Vec<Rc<dyn ReservationSummaryRule>>,
}

impl ReservationSummaryValidator {
    pub fn validate(&self, item: &ReservationSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReservationSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_summary_validator() -> ReservationSummaryValidator {
    ReservationSummaryValidator {
        rules: vec![
            Rc::new(ReservationSummaryEndsAtRangeRule { min: 80, max: 1000 }) as Rc<dyn ReservationSummaryRule>,
            Rc::new(ReservationSummaryAllowedAudienceRule {
                allowed: vec!["batched".to_string(), "manual".to_string()],
            }),
            Rc::new(ReservationSummaryVersionRangeRule { min: 64, max: 86400 }),
            Rc::new(ReservationSummaryLabelLengthRule { max_length: 86400 }),
            Rc::new(ReservationSummaryRequiredLabelRule {
                message: "reservation summary needs this field".to_string(),
            }),
            Rc::new(ReservationSummaryRequiredAudienceRule {
                message: "reservation summary needs a value".to_string(),
            }),
        ],
    }
}
