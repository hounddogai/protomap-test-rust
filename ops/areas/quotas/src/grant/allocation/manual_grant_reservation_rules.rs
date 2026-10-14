use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual grant reservation must pass. Check returns its problems, or none.
pub trait ManualGrantReservationRule: std::fmt::Debug {
    fn check(&self, item: &ManualGrantReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualGrantReservationCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualGrantReservationRule for ManualGrantReservationCostRangeRule {
    fn check(&self, item: &ManualGrantReservation) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualGrantReservationAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl ManualGrantReservationRule for ManualGrantReservationAllowedLabelRule {
    fn check(&self, item: &ManualGrantReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is expired", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualGrantReservationChannelLengthRule {
    pub max_length: i64,
}

impl ManualGrantReservationRule for ManualGrantReservationChannelLengthRule {
    fn check(&self, item: &ManualGrantReservation) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualGrantReservationRequiredChannelRule {
    pub message: String,
}

impl ManualGrantReservationRule for ManualGrantReservationRequiredChannelRule {
    fn check(&self, item: &ManualGrantReservation) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualGrantReservationRequiredCodeRule {
    pub message: String,
}

impl ManualGrantReservationRule for ManualGrantReservationRequiredCodeRule {
    fn check(&self, item: &ManualGrantReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualGrantReservationCodePrefixRule {
    pub prefix: String,
}

impl ManualGrantReservationRule for ManualGrantReservationCodePrefixRule {
    fn check(&self, item: &ManualGrantReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual grant reservation.
#[derive(Debug, Clone)]
pub struct ManualGrantReservationValidator {
    pub rules: Vec<Rc<dyn ManualGrantReservationRule>>,
}

impl ManualGrantReservationValidator {
    pub fn validate(&self, item: &ManualGrantReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualGrantReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualGrantReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_grant_reservation_validator() -> ManualGrantReservationValidator {
    ManualGrantReservationValidator {
        rules: vec![
            Rc::new(ManualGrantReservationCostRangeRule { min: 9.1, max: 2948.2 })
                as Rc<dyn ManualGrantReservationRule>,
            Rc::new(ManualGrantReservationAllowedLabelRule { allowed: Vec::new() }),
            Rc::new(ManualGrantReservationChannelLengthRule { max_length: 3600 }),
            Rc::new(ManualGrantReservationRequiredChannelRule {
                message: "manual grant reservation needs a value".to_string(),
            }),
            Rc::new(ManualGrantReservationRequiredCodeRule {
                message: "manual grant reservation needs this field".to_string(),
            }),
            Rc::new(ManualGrantReservationCodePrefixRule {
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
        let validator = default_manual_grant_reservation_validator();
        let item = new_manual_grant_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = ManualGrantReservationCostRangeRule { min: 7.2, max: 3049.9 };
        let mut item = new_manual_grant_reservation("x", "y");
        item.cost = 3051.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = ManualGrantReservationChannelLengthRule { max_length: 80 };
        let mut item = new_manual_grant_reservation("x", "y");
        item.channel = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = ManualGrantReservationRequiredChannelRule {
            message: "manual grant reservation needs an entry".to_string(),
        };
        let mut item = new_manual_grant_reservation("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualGrantReservationRequiredCodeRule {
            message: "manual grant reservation needs this field".to_string(),
        };
        let mut item = new_manual_grant_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
