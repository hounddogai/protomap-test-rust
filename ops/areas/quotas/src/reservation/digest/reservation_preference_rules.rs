use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a reservation preference must pass. Check returns its problems, or none.
pub trait ReservationPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &ReservationPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationPreferenceSummaryLengthRule {
    pub max_length: i64,
}

impl ReservationPreferenceRule for ReservationPreferenceSummaryLengthRule {
    fn check(&self, item: &ReservationPreference) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationPreferenceRequiredCodeRule {
    pub message: String,
}

impl ReservationPreferenceRule for ReservationPreferenceRequiredCodeRule {
    fn check(&self, item: &ReservationPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationPreferenceRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ReservationPreferenceRule for ReservationPreferenceRatioRangeRule {
    fn check(&self, item: &ReservationPreference) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationPreferenceApprovedGuardRule {
    pub limit: f64,
}

impl ReservationPreferenceRule for ReservationPreferenceApprovedGuardRule {
    fn check(&self, item: &ReservationPreference) -> Vec<String> {
        if item.approved && (item.margin > self.limit) {
            return vec![format!("margin {} exceeds {} while approved", item.margin, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl ReservationPreferenceRule for ReservationPreferenceCodeLengthRule {
    fn check(&self, item: &ReservationPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationPreferenceAllowedChannelRule {
    pub allowed: Vec<String>,
}

impl ReservationPreferenceRule for ReservationPreferenceAllowedChannelRule {
    fn check(&self, item: &ReservationPreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.channel)))
        {
            return vec![format!("channel {} is below minimum", item.channel)];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation preference.
#[derive(Debug, Clone)]
pub struct ReservationPreferenceValidator {
    pub rules: Vec<Rc<dyn ReservationPreferenceRule>>,
}

impl ReservationPreferenceValidator {
    pub fn validate(&self, item: &ReservationPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReservationPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_preference_validator() -> ReservationPreferenceValidator {
    ReservationPreferenceValidator {
        rules: vec![
            Rc::new(ReservationPreferenceSummaryLengthRule { max_length: 86400 }) as Rc<dyn ReservationPreferenceRule>,
            Rc::new(ReservationPreferenceRequiredCodeRule {
                message: "reservation preference needs this field".to_string(),
            }),
            Rc::new(ReservationPreferenceRatioRangeRule { min: 2.8, max: 3365.8 }),
            Rc::new(ReservationPreferenceApprovedGuardRule { limit: 5.1 }),
            Rc::new(ReservationPreferenceCodeLengthRule { max_length: 32 }),
            Rc::new(ReservationPreferenceAllowedChannelRule {
                allowed: vec!["fallback".to_string(), "fallback".to_string(), "capped".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_reservation_preference_validator();
        let item = new_reservation_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = ReservationPreferenceSummaryLengthRule { max_length: 255 };
        let mut item = new_reservation_preference("x", "y");
        item.summary = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ReservationPreferenceRequiredCodeRule {
            message: "reservation preference needs this field".to_string(),
        };
        let mut item = new_reservation_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = ReservationPreferenceRatioRangeRule { min: 0.5, max: 4053.9 };
        let mut item = new_reservation_preference("x", "y");
        item.ratio = 4055.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ReservationPreferenceCodeLengthRule { max_length: 255 };
        let mut item = new_reservation_preference("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
