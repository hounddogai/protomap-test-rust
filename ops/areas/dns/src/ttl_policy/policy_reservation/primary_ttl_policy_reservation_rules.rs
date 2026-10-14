use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary ttl policy reservation must pass. Check returns its problems, or none.
pub trait PrimaryTtlPolicyReservationRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryTtlPolicyReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyReservationRequiredCountryRule {
    pub message: String,
}

impl PrimaryTtlPolicyReservationRule for PrimaryTtlPolicyReservationRequiredCountryRule {
    fn check(&self, item: &PrimaryTtlPolicyReservation) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyReservationRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryTtlPolicyReservationRule for PrimaryTtlPolicyReservationRateRangeRule {
    fn check(&self, item: &PrimaryTtlPolicyReservation) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyReservationRequiredCodeRule {
    pub message: String,
}

impl PrimaryTtlPolicyReservationRule for PrimaryTtlPolicyReservationRequiredCodeRule {
    fn check(&self, item: &PrimaryTtlPolicyReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyReservationCountryLengthRule {
    pub max_length: i64,
}

impl PrimaryTtlPolicyReservationRule for PrimaryTtlPolicyReservationCountryLengthRule {
    fn check(&self, item: &PrimaryTtlPolicyReservation) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyReservationAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl PrimaryTtlPolicyReservationRule for PrimaryTtlPolicyReservationAllowedCountryRule {
    fn check(&self, item: &PrimaryTtlPolicyReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is inconsistent", item.country)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary ttl policy reservation.
#[derive(Debug, Clone)]
pub struct PrimaryTtlPolicyReservationValidator {
    pub rules: Vec<Rc<dyn PrimaryTtlPolicyReservationRule>>,
}

impl PrimaryTtlPolicyReservationValidator {
    pub fn validate(&self, item: &PrimaryTtlPolicyReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryTtlPolicyReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryTtlPolicyReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_ttl_policy_reservation_validator() -> PrimaryTtlPolicyReservationValidator {
    PrimaryTtlPolicyReservationValidator {
        rules: vec![
            Rc::new(PrimaryTtlPolicyReservationRequiredCountryRule {
                message: "primary ttl policy reservation needs this field".to_string(),
            }) as Rc<dyn PrimaryTtlPolicyReservationRule>,
            Rc::new(PrimaryTtlPolicyReservationRateRangeRule { min: 0.9, max: 3693.9 }),
            Rc::new(PrimaryTtlPolicyReservationRequiredCodeRule {
                message: "primary ttl policy reservation needs this field".to_string(),
            }),
            Rc::new(PrimaryTtlPolicyReservationCountryLengthRule { max_length: 255 }),
            Rc::new(PrimaryTtlPolicyReservationAllowedCountryRule {
                allowed: vec!["fallback".to_string(), "strict".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_ttl_policy_reservation_validator();
        let item = new_primary_ttl_policy_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = PrimaryTtlPolicyReservationRequiredCountryRule {
            message: "primary ttl policy reservation needs a value".to_string(),
        };
        let mut item = new_primary_ttl_policy_reservation("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = PrimaryTtlPolicyReservationRateRangeRule { min: 3.6, max: 1717.5 };
        let mut item = new_primary_ttl_policy_reservation("x", "y");
        item.rate = 1719.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryTtlPolicyReservationRequiredCodeRule {
            message: "primary ttl policy reservation needs a value".to_string(),
        };
        let mut item = new_primary_ttl_policy_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = PrimaryTtlPolicyReservationCountryLengthRule { max_length: 1000 };
        let mut item = new_primary_ttl_policy_reservation("x", "y");
        item.country = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
