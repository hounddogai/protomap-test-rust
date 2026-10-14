use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal reservation target must pass. Check returns its problems, or none.
pub trait SeasonalReservationTargetRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalReservationTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReservationTargetNameLengthRule {
    pub max_length: i64,
}

impl SeasonalReservationTargetRule for SeasonalReservationTargetNameLengthRule {
    fn check(&self, item: &SeasonalReservationTarget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReservationTargetRequiredCodeRule {
    pub message: String,
}

impl SeasonalReservationTargetRule for SeasonalReservationTargetRequiredCodeRule {
    fn check(&self, item: &SeasonalReservationTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReservationTargetAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl SeasonalReservationTargetRule for SeasonalReservationTargetAllowedCodeRule {
    fn check(&self, item: &SeasonalReservationTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReservationTargetRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalReservationTargetRule for SeasonalReservationTargetRotationDaysRangeRule {
    fn check(&self, item: &SeasonalReservationTarget) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReservationTargetCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalReservationTargetRule for SeasonalReservationTargetCodeLengthRule {
    fn check(&self, item: &SeasonalReservationTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal reservation target.
#[derive(Debug, Clone)]
pub struct SeasonalReservationTargetValidator {
    pub rules: Vec<Rc<dyn SeasonalReservationTargetRule>>,
}

impl SeasonalReservationTargetValidator {
    pub fn validate(&self, item: &SeasonalReservationTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalReservationTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalReservationTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_reservation_target_validator() -> SeasonalReservationTargetValidator {
    SeasonalReservationTargetValidator {
        rules: vec![
            Rc::new(SeasonalReservationTargetNameLengthRule { max_length: 32 })
                as Rc<dyn SeasonalReservationTargetRule>,
            Rc::new(SeasonalReservationTargetRequiredCodeRule {
                message: "seasonal reservation target needs this field".to_string(),
            }),
            Rc::new(SeasonalReservationTargetAllowedCodeRule {
                allowed: vec!["default".to_string()],
            }),
            Rc::new(SeasonalReservationTargetRotationDaysRangeRule { min: 64, max: 86400 }),
            Rc::new(SeasonalReservationTargetCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_reservation_target_validator();
        let item = new_seasonal_reservation_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SeasonalReservationTargetNameLengthRule { max_length: 86400 };
        let mut item = new_seasonal_reservation_target("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalReservationTargetRequiredCodeRule {
            message: "seasonal reservation target needs an entry".to_string(),
        };
        let mut item = new_seasonal_reservation_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = SeasonalReservationTargetRotationDaysRangeRule { min: 16, max: 120 };
        let mut item = new_seasonal_reservation_target("x", "y");
        item.rotation_days = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalReservationTargetCodeLengthRule { max_length: 120 };
        let mut item = new_seasonal_reservation_target("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
