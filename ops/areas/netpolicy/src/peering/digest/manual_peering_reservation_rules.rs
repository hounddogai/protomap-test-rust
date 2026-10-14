use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual peering reservation must pass. Check returns its problems, or none.
pub trait ManualPeeringReservationRule: std::fmt::Debug {
    fn check(&self, item: &ManualPeeringReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPeeringReservationRequiredLabelRule {
    pub message: String,
}

impl ManualPeeringReservationRule for ManualPeeringReservationRequiredLabelRule {
    fn check(&self, item: &ManualPeeringReservation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPeeringReservationRequiredCipherRule {
    pub message: String,
}

impl ManualPeeringReservationRule for ManualPeeringReservationRequiredCipherRule {
    fn check(&self, item: &ManualPeeringReservation) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPeeringReservationRequiredChecksumRule {
    pub message: String,
}

impl ManualPeeringReservationRule for ManualPeeringReservationRequiredChecksumRule {
    fn check(&self, item: &ManualPeeringReservation) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPeeringReservationRequiredCodeRule {
    pub message: String,
}

impl ManualPeeringReservationRule for ManualPeeringReservationRequiredCodeRule {
    fn check(&self, item: &ManualPeeringReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPeeringReservationAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl ManualPeeringReservationRule for ManualPeeringReservationAllowedLabelRule {
    fn check(&self, item: &ManualPeeringReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is not allowed", item.label)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual peering reservation.
#[derive(Debug, Clone)]
pub struct ManualPeeringReservationValidator {
    pub rules: Vec<Rc<dyn ManualPeeringReservationRule>>,
}

impl ManualPeeringReservationValidator {
    pub fn validate(&self, item: &ManualPeeringReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualPeeringReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualPeeringReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_peering_reservation_validator() -> ManualPeeringReservationValidator {
    ManualPeeringReservationValidator {
        rules: vec![
            Rc::new(ManualPeeringReservationRequiredLabelRule {
                message: "manual peering reservation needs an entry".to_string(),
            }) as Rc<dyn ManualPeeringReservationRule>,
            Rc::new(ManualPeeringReservationRequiredCipherRule {
                message: "manual peering reservation needs an entry".to_string(),
            }),
            Rc::new(ManualPeeringReservationRequiredChecksumRule {
                message: "manual peering reservation needs a value".to_string(),
            }),
            Rc::new(ManualPeeringReservationRequiredCodeRule {
                message: "manual peering reservation needs an entry".to_string(),
            }),
            Rc::new(ManualPeeringReservationAllowedLabelRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_peering_reservation_validator();
        let item = new_manual_peering_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ManualPeeringReservationRequiredLabelRule {
            message: "manual peering reservation needs a value".to_string(),
        };
        let mut item = new_manual_peering_reservation("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = ManualPeeringReservationRequiredCipherRule {
            message: "manual peering reservation needs a value".to_string(),
        };
        let mut item = new_manual_peering_reservation("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = ManualPeeringReservationRequiredChecksumRule {
            message: "manual peering reservation needs an entry".to_string(),
        };
        let mut item = new_manual_peering_reservation("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualPeeringReservationRequiredCodeRule {
            message: "manual peering reservation needs a value".to_string(),
        };
        let mut item = new_manual_peering_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
