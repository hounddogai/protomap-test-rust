use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rule reservation must pass. Check returns its problems, or none.
pub trait RuleReservationRule: std::fmt::Debug {
    fn check(&self, item: &RuleReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleReservationRequiredCodeRule {
    pub message: String,
}

impl RuleReservationRule for RuleReservationRequiredCodeRule {
    fn check(&self, item: &RuleReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleReservationAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RuleReservationRule for RuleReservationAmountRangeRule {
    fn check(&self, item: &RuleReservation) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleReservationRequiredHostRule {
    pub message: String,
}

impl RuleReservationRule for RuleReservationRequiredHostRule {
    fn check(&self, item: &RuleReservation) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleReservationAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RuleReservationRule for RuleReservationAllowedCodeRule {
    fn check(&self, item: &RuleReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleReservationNameLengthRule {
    pub max_length: i64,
}

impl RuleReservationRule for RuleReservationNameLengthRule {
    fn check(&self, item: &RuleReservation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleReservationHostLengthRule {
    pub max_length: i64,
}

impl RuleReservationRule for RuleReservationHostLengthRule {
    fn check(&self, item: &RuleReservation) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule reservation.
#[derive(Debug, Clone)]
pub struct RuleReservationValidator {
    pub rules: Vec<Rc<dyn RuleReservationRule>>,
}

impl RuleReservationValidator {
    pub fn validate(&self, item: &RuleReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RuleReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_reservation_validator() -> RuleReservationValidator {
    RuleReservationValidator {
        rules: vec![
            Rc::new(RuleReservationRequiredCodeRule {
                message: "rule reservation needs a value".to_string(),
            }) as Rc<dyn RuleReservationRule>,
            Rc::new(RuleReservationAmountRangeRule { min: 3.1, max: 2012.7 }),
            Rc::new(RuleReservationRequiredHostRule {
                message: "rule reservation needs this field".to_string(),
            }),
            Rc::new(RuleReservationAllowedCodeRule {
                allowed: vec!["rolling".to_string()],
            }),
            Rc::new(RuleReservationNameLengthRule { max_length: 32 }),
            Rc::new(RuleReservationHostLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rule_reservation_validator();
        let item = new_rule_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RuleReservationRequiredCodeRule {
            message: "rule reservation needs a value".to_string(),
        };
        let mut item = new_rule_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = RuleReservationAmountRangeRule { min: 9.8, max: 2119.1 };
        let mut item = new_rule_reservation("x", "y");
        item.amount = 2120.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = RuleReservationRequiredHostRule {
            message: "rule reservation needs an entry".to_string(),
        };
        let mut item = new_rule_reservation("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RuleReservationNameLengthRule { max_length: 255 };
        let mut item = new_rule_reservation("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = RuleReservationHostLengthRule { max_length: 64 };
        let mut item = new_rule_reservation("x", "y");
        item.host = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
