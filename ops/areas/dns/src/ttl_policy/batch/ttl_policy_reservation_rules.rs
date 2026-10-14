use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a ttl policy reservation must pass. Check returns its problems, or none.
pub trait TtlPolicyReservationRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyReservationRequiredChecksumRule {
    pub message: String,
}

impl TtlPolicyReservationRule for TtlPolicyReservationRequiredChecksumRule {
    fn check(&self, item: &TtlPolicyReservation) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyReservationRequiredTitleRule {
    pub message: String,
}

impl TtlPolicyReservationRule for TtlPolicyReservationRequiredTitleRule {
    fn check(&self, item: &TtlPolicyReservation) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyReservationRequiredCodeRule {
    pub message: String,
}

impl TtlPolicyReservationRule for TtlPolicyReservationRequiredCodeRule {
    fn check(&self, item: &TtlPolicyReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy reservation.
#[derive(Debug, Clone)]
pub struct TtlPolicyReservationValidator {
    pub rules: Vec<Rc<dyn TtlPolicyReservationRule>>,
}

impl TtlPolicyReservationValidator {
    pub fn validate(&self, item: &TtlPolicyReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TtlPolicyReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_reservation_validator() -> TtlPolicyReservationValidator {
    TtlPolicyReservationValidator {
        rules: vec![
            Rc::new(TtlPolicyReservationRequiredChecksumRule {
                message: "ttl policy reservation needs an entry".to_string(),
            }) as Rc<dyn TtlPolicyReservationRule>,
            Rc::new(TtlPolicyReservationRequiredTitleRule {
                message: "ttl policy reservation needs this field".to_string(),
            }),
            Rc::new(TtlPolicyReservationRequiredCodeRule {
                message: "ttl policy reservation needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_ttl_policy_reservation_validator();
        let item = new_ttl_policy_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = TtlPolicyReservationRequiredChecksumRule {
            message: "ttl policy reservation needs an entry".to_string(),
        };
        let mut item = new_ttl_policy_reservation("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TtlPolicyReservationRequiredTitleRule {
            message: "ttl policy reservation needs this field".to_string(),
        };
        let mut item = new_ttl_policy_reservation("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TtlPolicyReservationRequiredCodeRule {
            message: "ttl policy reservation needs this field".to_string(),
        };
        let mut item = new_ttl_policy_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
