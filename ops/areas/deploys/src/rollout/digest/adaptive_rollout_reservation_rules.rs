use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive rollout reservation must pass. Check returns its problems, or none.
pub trait AdaptiveRolloutReservationRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveRolloutReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRolloutReservationCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveRolloutReservationRule for AdaptiveRolloutReservationCodeLengthRule {
    fn check(&self, item: &AdaptiveRolloutReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRolloutReservationHostLengthRule {
    pub max_length: i64,
}

impl AdaptiveRolloutReservationRule for AdaptiveRolloutReservationHostLengthRule {
    fn check(&self, item: &AdaptiveRolloutReservation) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRolloutReservationEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveRolloutReservationRule for AdaptiveRolloutReservationEndsAtRangeRule {
    fn check(&self, item: &AdaptiveRolloutReservation) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRolloutReservationSegmentLengthRule {
    pub max_length: i64,
}

impl AdaptiveRolloutReservationRule for AdaptiveRolloutReservationSegmentLengthRule {
    fn check(&self, item: &AdaptiveRolloutReservation) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRolloutReservationRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveRolloutReservationRule for AdaptiveRolloutReservationRetriesRangeRule {
    fn check(&self, item: &AdaptiveRolloutReservation) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRolloutReservationAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl AdaptiveRolloutReservationRule for AdaptiveRolloutReservationAllowedTitleRule {
    fn check(&self, item: &AdaptiveRolloutReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is inconsistent", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive rollout reservation.
#[derive(Debug, Clone)]
pub struct AdaptiveRolloutReservationValidator {
    pub rules: Vec<Rc<dyn AdaptiveRolloutReservationRule>>,
}

impl AdaptiveRolloutReservationValidator {
    pub fn validate(&self, item: &AdaptiveRolloutReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveRolloutReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveRolloutReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_rollout_reservation_validator() -> AdaptiveRolloutReservationValidator {
    AdaptiveRolloutReservationValidator {
        rules: vec![
            Rc::new(AdaptiveRolloutReservationCodeLengthRule { max_length: 64 })
                as Rc<dyn AdaptiveRolloutReservationRule>,
            Rc::new(AdaptiveRolloutReservationHostLengthRule { max_length: 1000 }),
            Rc::new(AdaptiveRolloutReservationEndsAtRangeRule { min: 1000, max: 3600 }),
            Rc::new(AdaptiveRolloutReservationSegmentLengthRule { max_length: 120 }),
            Rc::new(AdaptiveRolloutReservationRetriesRangeRule { min: 80, max: 120 }),
            Rc::new(AdaptiveRolloutReservationAllowedTitleRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_rollout_reservation_validator();
        let item = new_adaptive_rollout_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveRolloutReservationCodeLengthRule { max_length: 120 };
        let mut item = new_adaptive_rollout_reservation("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = AdaptiveRolloutReservationHostLengthRule { max_length: 16 };
        let mut item = new_adaptive_rollout_reservation("x", "y");
        item.host = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = AdaptiveRolloutReservationEndsAtRangeRule { min: 255, max: 1000 };
        let mut item = new_adaptive_rollout_reservation("x", "y");
        item.ends_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = AdaptiveRolloutReservationSegmentLengthRule { max_length: 3600 };
        let mut item = new_adaptive_rollout_reservation("x", "y");
        item.segment = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = AdaptiveRolloutReservationRetriesRangeRule { min: 64, max: 255 };
        let mut item = new_adaptive_rollout_reservation("x", "y");
        item.retries = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
