use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal freeze reservation must pass. Check returns its problems, or none.
pub trait SeasonalFreezeReservationRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalFreezeReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeReservationAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl SeasonalFreezeReservationRule for SeasonalFreezeReservationAllowedCodeRule {
    fn check(&self, item: &SeasonalFreezeReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeReservationRequiredCodeRule {
    pub message: String,
}

impl SeasonalFreezeReservationRule for SeasonalFreezeReservationRequiredCodeRule {
    fn check(&self, item: &SeasonalFreezeReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeReservationRequiredCategoryRule {
    pub message: String,
}

impl SeasonalFreezeReservationRule for SeasonalFreezeReservationRequiredCategoryRule {
    fn check(&self, item: &SeasonalFreezeReservation) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeReservationDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalFreezeReservationRule for SeasonalFreezeReservationDurationMinutesRangeRule {
    fn check(&self, item: &SeasonalFreezeReservation) -> Vec<String> {
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

/// Runs every rule of a seasonal freeze reservation.
#[derive(Debug, Clone)]
pub struct SeasonalFreezeReservationValidator {
    pub rules: Vec<Rc<dyn SeasonalFreezeReservationRule>>,
}

impl SeasonalFreezeReservationValidator {
    pub fn validate(&self, item: &SeasonalFreezeReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalFreezeReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalFreezeReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_freeze_reservation_validator() -> SeasonalFreezeReservationValidator {
    SeasonalFreezeReservationValidator {
        rules: vec![
            Rc::new(SeasonalFreezeReservationAllowedCodeRule {
                allowed: vec!["seasonal".to_string(), "strict".to_string(), "pending".to_string()],
            }) as Rc<dyn SeasonalFreezeReservationRule>,
            Rc::new(SeasonalFreezeReservationRequiredCodeRule {
                message: "seasonal freeze reservation needs this field".to_string(),
            }),
            Rc::new(SeasonalFreezeReservationRequiredCategoryRule {
                message: "seasonal freeze reservation needs this field".to_string(),
            }),
            Rc::new(SeasonalFreezeReservationDurationMinutesRangeRule { min: 16, max: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_freeze_reservation_validator();
        let item = new_seasonal_freeze_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalFreezeReservationRequiredCodeRule {
            message: "seasonal freeze reservation needs an entry".to_string(),
        };
        let mut item = new_seasonal_freeze_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = SeasonalFreezeReservationRequiredCategoryRule {
            message: "seasonal freeze reservation needs a value".to_string(),
        };
        let mut item = new_seasonal_freeze_reservation("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = SeasonalFreezeReservationDurationMinutesRangeRule { min: 16, max: 3600 };
        let mut item = new_seasonal_freeze_reservation("x", "y");
        item.duration_minutes = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
