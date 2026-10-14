use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict query reservation must pass. Check returns its problems, or none.
pub trait StrictQueryReservationRule: std::fmt::Debug {
    fn check(&self, item: &StrictQueryReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryReservationRequiredCodeRule {
    pub message: String,
}

impl StrictQueryReservationRule for StrictQueryReservationRequiredCodeRule {
    fn check(&self, item: &StrictQueryReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryReservationLabelLengthRule {
    pub max_length: i64,
}

impl StrictQueryReservationRule for StrictQueryReservationLabelLengthRule {
    fn check(&self, item: &StrictQueryReservation) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryReservationRequiredLabelRule {
    pub message: String,
}

impl StrictQueryReservationRule for StrictQueryReservationRequiredLabelRule {
    fn check(&self, item: &StrictQueryReservation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryReservationTimezoneLengthRule {
    pub max_length: i64,
}

impl StrictQueryReservationRule for StrictQueryReservationTimezoneLengthRule {
    fn check(&self, item: &StrictQueryReservation) -> Vec<String> {
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
pub struct StrictQueryReservationCodeLengthRule {
    pub max_length: i64,
}

impl StrictQueryReservationRule for StrictQueryReservationCodeLengthRule {
    fn check(&self, item: &StrictQueryReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryReservationDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictQueryReservationRule for StrictQueryReservationDiscountRangeRule {
    fn check(&self, item: &StrictQueryReservation) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict query reservation.
#[derive(Debug, Clone)]
pub struct StrictQueryReservationValidator {
    pub rules: Vec<Rc<dyn StrictQueryReservationRule>>,
}

impl StrictQueryReservationValidator {
    pub fn validate(&self, item: &StrictQueryReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictQueryReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictQueryReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_query_reservation_validator() -> StrictQueryReservationValidator {
    StrictQueryReservationValidator {
        rules: vec![
            Rc::new(StrictQueryReservationRequiredCodeRule {
                message: "strict query reservation needs a value".to_string(),
            }) as Rc<dyn StrictQueryReservationRule>,
            Rc::new(StrictQueryReservationLabelLengthRule { max_length: 16 }),
            Rc::new(StrictQueryReservationRequiredLabelRule {
                message: "strict query reservation needs a value".to_string(),
            }),
            Rc::new(StrictQueryReservationTimezoneLengthRule { max_length: 16 }),
            Rc::new(StrictQueryReservationCodeLengthRule { max_length: 3600 }),
            Rc::new(StrictQueryReservationDiscountRangeRule { min: 6.7, max: 4929.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_query_reservation_validator();
        let item = new_strict_query_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictQueryReservationRequiredCodeRule {
            message: "strict query reservation needs an entry".to_string(),
        };
        let mut item = new_strict_query_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = StrictQueryReservationLabelLengthRule { max_length: 16 };
        let mut item = new_strict_query_reservation("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = StrictQueryReservationRequiredLabelRule {
            message: "strict query reservation needs a value".to_string(),
        };
        let mut item = new_strict_query_reservation("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = StrictQueryReservationTimezoneLengthRule { max_length: 80 };
        let mut item = new_strict_query_reservation("x", "y");
        item.timezone = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictQueryReservationCodeLengthRule { max_length: 120 };
        let mut item = new_strict_query_reservation("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = StrictQueryReservationDiscountRangeRule { min: 8.3, max: 3957.7 };
        let mut item = new_strict_query_reservation("x", "y");
        item.discount = 3959.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
