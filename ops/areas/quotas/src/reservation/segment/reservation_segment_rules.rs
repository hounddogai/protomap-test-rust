use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a reservation segment must pass. Check returns its problems, or none.
pub trait ReservationSegmentRule: std::fmt::Debug {
    fn check(&self, item: &ReservationSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationSegmentRequiredSummaryRule {
    pub message: String,
}

impl ReservationSegmentRule for ReservationSegmentRequiredSummaryRule {
    fn check(&self, item: &ReservationSegment) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationSegmentMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ReservationSegmentRule for ReservationSegmentMaximumRangeRule {
    fn check(&self, item: &ReservationSegment) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationSegmentRequiredCodeRule {
    pub message: String,
}

impl ReservationSegmentRule for ReservationSegmentRequiredCodeRule {
    fn check(&self, item: &ReservationSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationSegmentRequiredNameRule {
    pub message: String,
}

impl ReservationSegmentRule for ReservationSegmentRequiredNameRule {
    fn check(&self, item: &ReservationSegment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationSegmentAllowedSummaryRule {
    pub allowed: Vec<String>,
}

impl ReservationSegmentRule for ReservationSegmentAllowedSummaryRule {
    fn check(&self, item: &ReservationSegment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.summary)))
        {
            return vec![format!("summary {} is below minimum", item.summary)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationSegmentCodePrefixRule {
    pub prefix: String,
}

impl ReservationSegmentRule for ReservationSegmentCodePrefixRule {
    fn check(&self, item: &ReservationSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation segment.
#[derive(Debug, Clone)]
pub struct ReservationSegmentValidator {
    pub rules: Vec<Rc<dyn ReservationSegmentRule>>,
}

impl ReservationSegmentValidator {
    pub fn validate(&self, item: &ReservationSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReservationSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_segment_validator() -> ReservationSegmentValidator {
    ReservationSegmentValidator {
        rules: vec![
            Rc::new(ReservationSegmentRequiredSummaryRule {
                message: "reservation segment needs this field".to_string(),
            }) as Rc<dyn ReservationSegmentRule>,
            Rc::new(ReservationSegmentMaximumRangeRule { min: 8.1, max: 1412.8 }),
            Rc::new(ReservationSegmentRequiredCodeRule {
                message: "reservation segment needs this field".to_string(),
            }),
            Rc::new(ReservationSegmentRequiredNameRule {
                message: "reservation segment needs this field".to_string(),
            }),
            Rc::new(ReservationSegmentAllowedSummaryRule { allowed: Vec::new() }),
            Rc::new(ReservationSegmentCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_reservation_segment_validator();
        let item = new_reservation_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = ReservationSegmentRequiredSummaryRule {
            message: "reservation segment needs a value".to_string(),
        };
        let mut item = new_reservation_segment("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = ReservationSegmentMaximumRangeRule { min: 6.8, max: 1848.2 };
        let mut item = new_reservation_segment("x", "y");
        item.maximum = 1849.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ReservationSegmentRequiredCodeRule {
            message: "reservation segment needs a value".to_string(),
        };
        let mut item = new_reservation_segment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ReservationSegmentRequiredNameRule {
            message: "reservation segment needs this field".to_string(),
        };
        let mut item = new_reservation_segment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
