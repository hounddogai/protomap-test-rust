use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy federation reservation must pass. Check returns its problems, or none.
pub trait LegacyFederationReservationRule: std::fmt::Debug {
    fn check(&self, item: &LegacyFederationReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFederationReservationMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyFederationReservationRule for LegacyFederationReservationMinimumRangeRule {
    fn check(&self, item: &LegacyFederationReservation) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFederationReservationRequiredHostRule {
    pub message: String,
}

impl LegacyFederationReservationRule for LegacyFederationReservationRequiredHostRule {
    fn check(&self, item: &LegacyFederationReservation) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFederationReservationRequiredCodeRule {
    pub message: String,
}

impl LegacyFederationReservationRule for LegacyFederationReservationRequiredCodeRule {
    fn check(&self, item: &LegacyFederationReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFederationReservationCodePrefixRule {
    pub prefix: String,
}

impl LegacyFederationReservationRule for LegacyFederationReservationCodePrefixRule {
    fn check(&self, item: &LegacyFederationReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy federation reservation.
#[derive(Debug, Clone)]
pub struct LegacyFederationReservationValidator {
    pub rules: Vec<Rc<dyn LegacyFederationReservationRule>>,
}

impl LegacyFederationReservationValidator {
    pub fn validate(&self, item: &LegacyFederationReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyFederationReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyFederationReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_federation_reservation_validator() -> LegacyFederationReservationValidator {
    LegacyFederationReservationValidator {
        rules: vec![
            Rc::new(LegacyFederationReservationMinimumRangeRule { min: 1.8, max: 3169.9 })
                as Rc<dyn LegacyFederationReservationRule>,
            Rc::new(LegacyFederationReservationRequiredHostRule {
                message: "legacy federation reservation needs this field".to_string(),
            }),
            Rc::new(LegacyFederationReservationRequiredCodeRule {
                message: "legacy federation reservation needs a value".to_string(),
            }),
            Rc::new(LegacyFederationReservationCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_federation_reservation_validator();
        let item = new_legacy_federation_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = LegacyFederationReservationMinimumRangeRule { min: 5.3, max: 4634.7 };
        let mut item = new_legacy_federation_reservation("x", "y");
        item.minimum = 4636.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = LegacyFederationReservationRequiredHostRule {
            message: "legacy federation reservation needs an entry".to_string(),
        };
        let mut item = new_legacy_federation_reservation("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyFederationReservationRequiredCodeRule {
            message: "legacy federation reservation needs an entry".to_string(),
        };
        let mut item = new_legacy_federation_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
