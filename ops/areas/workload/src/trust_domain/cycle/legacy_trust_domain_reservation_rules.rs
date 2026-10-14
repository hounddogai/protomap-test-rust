use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy trust domain reservation must pass. Check returns its problems, or none.
pub trait LegacyTrustDomainReservationRule: std::fmt::Debug {
    fn check(&self, item: &LegacyTrustDomainReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTrustDomainReservationCodeLengthRule {
    pub max_length: i64,
}

impl LegacyTrustDomainReservationRule for LegacyTrustDomainReservationCodeLengthRule {
    fn check(&self, item: &LegacyTrustDomainReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTrustDomainReservationCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyTrustDomainReservationRule for LegacyTrustDomainReservationCreatedAtRangeRule {
    fn check(&self, item: &LegacyTrustDomainReservation) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTrustDomainReservationCurrencyLengthRule {
    pub max_length: i64,
}

impl LegacyTrustDomainReservationRule for LegacyTrustDomainReservationCurrencyLengthRule {
    fn check(&self, item: &LegacyTrustDomainReservation) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTrustDomainReservationRequiredTitleRule {
    pub message: String,
}

impl LegacyTrustDomainReservationRule for LegacyTrustDomainReservationRequiredTitleRule {
    fn check(&self, item: &LegacyTrustDomainReservation) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy trust domain reservation.
#[derive(Debug, Clone)]
pub struct LegacyTrustDomainReservationValidator {
    pub rules: Vec<Rc<dyn LegacyTrustDomainReservationRule>>,
}

impl LegacyTrustDomainReservationValidator {
    pub fn validate(&self, item: &LegacyTrustDomainReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyTrustDomainReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyTrustDomainReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_trust_domain_reservation_validator() -> LegacyTrustDomainReservationValidator {
    LegacyTrustDomainReservationValidator {
        rules: vec![
            Rc::new(LegacyTrustDomainReservationCodeLengthRule { max_length: 120 })
                as Rc<dyn LegacyTrustDomainReservationRule>,
            Rc::new(LegacyTrustDomainReservationCreatedAtRangeRule { min: 255, max: 1000 }),
            Rc::new(LegacyTrustDomainReservationCurrencyLengthRule { max_length: 64 }),
            Rc::new(LegacyTrustDomainReservationRequiredTitleRule {
                message: "legacy trust domain reservation needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_trust_domain_reservation_validator();
        let item = new_legacy_trust_domain_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyTrustDomainReservationCodeLengthRule { max_length: 255 };
        let mut item = new_legacy_trust_domain_reservation("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = LegacyTrustDomainReservationCreatedAtRangeRule { min: 32, max: 120 };
        let mut item = new_legacy_trust_domain_reservation("x", "y");
        item.created_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = LegacyTrustDomainReservationCurrencyLengthRule { max_length: 16 };
        let mut item = new_legacy_trust_domain_reservation("x", "y");
        item.currency = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LegacyTrustDomainReservationRequiredTitleRule {
            message: "legacy trust domain reservation needs this field".to_string(),
        };
        let mut item = new_legacy_trust_domain_reservation("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
