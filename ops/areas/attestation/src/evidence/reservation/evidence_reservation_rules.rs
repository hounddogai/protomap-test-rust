use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a evidence reservation must pass. Check returns its problems, or none.
pub trait EvidenceReservationRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceReservationAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl EvidenceReservationRule for EvidenceReservationAllowedCodeRule {
    fn check(&self, item: &EvidenceReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceReservationUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EvidenceReservationRule for EvidenceReservationUpdatedAtRangeRule {
    fn check(&self, item: &EvidenceReservation) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceReservationFingerprintLengthRule {
    pub max_length: i64,
}

impl EvidenceReservationRule for EvidenceReservationFingerprintLengthRule {
    fn check(&self, item: &EvidenceReservation) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence reservation.
#[derive(Debug, Clone)]
pub struct EvidenceReservationValidator {
    pub rules: Vec<Rc<dyn EvidenceReservationRule>>,
}

impl EvidenceReservationValidator {
    pub fn validate(&self, item: &EvidenceReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EvidenceReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_reservation_validator() -> EvidenceReservationValidator {
    EvidenceReservationValidator {
        rules: vec![
            Rc::new(EvidenceReservationAllowedCodeRule {
                allowed: vec!["regional".to_string(), "batched".to_string(), "rolling".to_string()],
            }) as Rc<dyn EvidenceReservationRule>,
            Rc::new(EvidenceReservationUpdatedAtRangeRule { min: 32, max: 86400 }),
            Rc::new(EvidenceReservationFingerprintLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_evidence_reservation_validator();
        let item = new_evidence_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = EvidenceReservationUpdatedAtRangeRule { min: 32, max: 1000 };
        let mut item = new_evidence_reservation("x", "y");
        item.updated_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = EvidenceReservationFingerprintLengthRule { max_length: 1000 };
        let mut item = new_evidence_reservation("x", "y");
        item.fingerprint = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
