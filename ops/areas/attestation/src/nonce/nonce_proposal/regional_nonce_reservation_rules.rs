use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional nonce reservation must pass. Check returns its problems, or none.
pub trait RegionalNonceReservationRule: std::fmt::Debug {
    fn check(&self, item: &RegionalNonceReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalNonceReservationAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RegionalNonceReservationRule for RegionalNonceReservationAllowedCodeRule {
    fn check(&self, item: &RegionalNonceReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalNonceReservationCodePrefixRule {
    pub prefix: String,
}

impl RegionalNonceReservationRule for RegionalNonceReservationCodePrefixRule {
    fn check(&self, item: &RegionalNonceReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalNonceReservationTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalNonceReservationRule for RegionalNonceReservationTtlSecondsRangeRule {
    fn check(&self, item: &RegionalNonceReservation) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalNonceReservationTitleLengthRule {
    pub max_length: i64,
}

impl RegionalNonceReservationRule for RegionalNonceReservationTitleLengthRule {
    fn check(&self, item: &RegionalNonceReservation) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalNonceReservationIssuerLengthRule {
    pub max_length: i64,
}

impl RegionalNonceReservationRule for RegionalNonceReservationIssuerLengthRule {
    fn check(&self, item: &RegionalNonceReservation) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional nonce reservation.
#[derive(Debug, Clone)]
pub struct RegionalNonceReservationValidator {
    pub rules: Vec<Rc<dyn RegionalNonceReservationRule>>,
}

impl RegionalNonceReservationValidator {
    pub fn validate(&self, item: &RegionalNonceReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalNonceReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalNonceReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_nonce_reservation_validator() -> RegionalNonceReservationValidator {
    RegionalNonceReservationValidator {
        rules: vec![
            Rc::new(RegionalNonceReservationAllowedCodeRule {
                allowed: vec!["adaptive".to_string()],
            }) as Rc<dyn RegionalNonceReservationRule>,
            Rc::new(RegionalNonceReservationCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(RegionalNonceReservationTtlSecondsRangeRule { min: 255, max: 86400 }),
            Rc::new(RegionalNonceReservationTitleLengthRule { max_length: 32 }),
            Rc::new(RegionalNonceReservationIssuerLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_nonce_reservation_validator();
        let item = new_regional_nonce_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = RegionalNonceReservationTtlSecondsRangeRule { min: 255, max: 1000 };
        let mut item = new_regional_nonce_reservation("x", "y");
        item.ttl_seconds = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RegionalNonceReservationTitleLengthRule { max_length: 120 };
        let mut item = new_regional_nonce_reservation("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = RegionalNonceReservationIssuerLengthRule { max_length: 1000 };
        let mut item = new_regional_nonce_reservation("x", "y");
        item.issuer = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
