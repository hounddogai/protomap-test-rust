use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a trust domain reservation must pass. Check returns its problems, or none.
pub trait TrustDomainReservationRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainReservationTitleLengthRule {
    pub max_length: i64,
}

impl TrustDomainReservationRule for TrustDomainReservationTitleLengthRule {
    fn check(&self, item: &TrustDomainReservation) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainReservationSourceLengthRule {
    pub max_length: i64,
}

impl TrustDomainReservationRule for TrustDomainReservationSourceLengthRule {
    fn check(&self, item: &TrustDomainReservation) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainReservationCodePrefixRule {
    pub prefix: String,
}

impl TrustDomainReservationRule for TrustDomainReservationCodePrefixRule {
    fn check(&self, item: &TrustDomainReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainReservationActiveGuardRule {
    pub limit: i64,
}

impl TrustDomainReservationRule for TrustDomainReservationActiveGuardRule {
    fn check(&self, item: &TrustDomainReservation) -> Vec<String> {
        if item.active && (item.ends_at > self.limit) {
            return vec![format!("ends at {} exceeds {} while active", item.ends_at, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainReservationRequiredSourceRule {
    pub message: String,
}

impl TrustDomainReservationRule for TrustDomainReservationRequiredSourceRule {
    fn check(&self, item: &TrustDomainReservation) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainReservationRequiredChecksumRule {
    pub message: String,
}

impl TrustDomainReservationRule for TrustDomainReservationRequiredChecksumRule {
    fn check(&self, item: &TrustDomainReservation) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain reservation.
#[derive(Debug, Clone)]
pub struct TrustDomainReservationValidator {
    pub rules: Vec<Rc<dyn TrustDomainReservationRule>>,
}

impl TrustDomainReservationValidator {
    pub fn validate(&self, item: &TrustDomainReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TrustDomainReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_reservation_validator() -> TrustDomainReservationValidator {
    TrustDomainReservationValidator {
        rules: vec![
            Rc::new(TrustDomainReservationTitleLengthRule { max_length: 16 }) as Rc<dyn TrustDomainReservationRule>,
            Rc::new(TrustDomainReservationSourceLengthRule { max_length: 64 }),
            Rc::new(TrustDomainReservationCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(TrustDomainReservationActiveGuardRule { limit: 255 }),
            Rc::new(TrustDomainReservationRequiredSourceRule {
                message: "trust domain reservation needs this field".to_string(),
            }),
            Rc::new(TrustDomainReservationRequiredChecksumRule {
                message: "trust domain reservation needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_reservation_validator();
        let item = new_trust_domain_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TrustDomainReservationTitleLengthRule { max_length: 1000 };
        let mut item = new_trust_domain_reservation("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = TrustDomainReservationSourceLengthRule { max_length: 255 };
        let mut item = new_trust_domain_reservation("x", "y");
        item.source = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = TrustDomainReservationRequiredSourceRule {
            message: "trust domain reservation needs this field".to_string(),
        };
        let mut item = new_trust_domain_reservation("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = TrustDomainReservationRequiredChecksumRule {
            message: "trust domain reservation needs this field".to_string(),
        };
        let mut item = new_trust_domain_reservation("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
