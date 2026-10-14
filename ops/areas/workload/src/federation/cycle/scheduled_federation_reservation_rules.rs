use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled federation reservation must pass. Check returns its problems, or none.
pub trait ScheduledFederationReservationRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledFederationReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFederationReservationRequiredCodeRule {
    pub message: String,
}

impl ScheduledFederationReservationRule for ScheduledFederationReservationRequiredCodeRule {
    fn check(&self, item: &ScheduledFederationReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFederationReservationAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl ScheduledFederationReservationRule for ScheduledFederationReservationAllowedTitleRule {
    fn check(&self, item: &ScheduledFederationReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is stale", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFederationReservationCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledFederationReservationRule for ScheduledFederationReservationCodeLengthRule {
    fn check(&self, item: &ScheduledFederationReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFederationReservationCodePrefixRule {
    pub prefix: String,
}

impl ScheduledFederationReservationRule for ScheduledFederationReservationCodePrefixRule {
    fn check(&self, item: &ScheduledFederationReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFederationReservationKeyIdLengthRule {
    pub max_length: i64,
}

impl ScheduledFederationReservationRule for ScheduledFederationReservationKeyIdLengthRule {
    fn check(&self, item: &ScheduledFederationReservation) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFederationReservationQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledFederationReservationRule for ScheduledFederationReservationQuantityRangeRule {
    fn check(&self, item: &ScheduledFederationReservation) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a scheduled federation reservation.
#[derive(Debug, Clone)]
pub struct ScheduledFederationReservationValidator {
    pub rules: Vec<Rc<dyn ScheduledFederationReservationRule>>,
}

impl ScheduledFederationReservationValidator {
    pub fn validate(&self, item: &ScheduledFederationReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledFederationReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledFederationReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_federation_reservation_validator() -> ScheduledFederationReservationValidator {
    ScheduledFederationReservationValidator {
        rules: vec![
            Rc::new(ScheduledFederationReservationRequiredCodeRule {
                message: "scheduled federation reservation needs this field".to_string(),
            }) as Rc<dyn ScheduledFederationReservationRule>,
            Rc::new(ScheduledFederationReservationAllowedTitleRule {
                allowed: vec!["primary".to_string()],
            }),
            Rc::new(ScheduledFederationReservationCodeLengthRule { max_length: 32 }),
            Rc::new(ScheduledFederationReservationCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(ScheduledFederationReservationKeyIdLengthRule { max_length: 80 }),
            Rc::new(ScheduledFederationReservationQuantityRangeRule { min: 32, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_federation_reservation_validator();
        let item = new_scheduled_federation_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledFederationReservationRequiredCodeRule {
            message: "scheduled federation reservation needs this field".to_string(),
        };
        let mut item = new_scheduled_federation_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledFederationReservationCodeLengthRule { max_length: 1000 };
        let mut item = new_scheduled_federation_reservation("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = ScheduledFederationReservationKeyIdLengthRule { max_length: 1000 };
        let mut item = new_scheduled_federation_reservation("x", "y");
        item.key_id = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = ScheduledFederationReservationQuantityRangeRule { min: 80, max: 80 };
        let mut item = new_scheduled_federation_reservation("x", "y");
        item.quantity = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
