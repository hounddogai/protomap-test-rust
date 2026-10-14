use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a reservation digest must pass. Check returns its problems, or none.
pub trait ReservationDigestRule: std::fmt::Debug {
    fn check(&self, item: &ReservationDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationDigestIssuerLengthRule {
    pub max_length: i64,
}

impl ReservationDigestRule for ReservationDigestIssuerLengthRule {
    fn check(&self, item: &ReservationDigest) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationDigestRequiredCodeRule {
    pub message: String,
}

impl ReservationDigestRule for ReservationDigestRequiredCodeRule {
    fn check(&self, item: &ReservationDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationDigestChannelLengthRule {
    pub max_length: i64,
}

impl ReservationDigestRule for ReservationDigestChannelLengthRule {
    fn check(&self, item: &ReservationDigest) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationDigestRequiredIssuerRule {
    pub message: String,
}

impl ReservationDigestRule for ReservationDigestRequiredIssuerRule {
    fn check(&self, item: &ReservationDigest) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation digest.
#[derive(Debug, Clone)]
pub struct ReservationDigestValidator {
    pub rules: Vec<Rc<dyn ReservationDigestRule>>,
}

impl ReservationDigestValidator {
    pub fn validate(&self, item: &ReservationDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReservationDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_digest_validator() -> ReservationDigestValidator {
    ReservationDigestValidator {
        rules: vec![
            Rc::new(ReservationDigestIssuerLengthRule { max_length: 80 }) as Rc<dyn ReservationDigestRule>,
            Rc::new(ReservationDigestRequiredCodeRule {
                message: "reservation digest needs this field".to_string(),
            }),
            Rc::new(ReservationDigestChannelLengthRule { max_length: 64 }),
            Rc::new(ReservationDigestRequiredIssuerRule {
                message: "reservation digest needs an entry".to_string(),
            }),
        ],
    }
}
