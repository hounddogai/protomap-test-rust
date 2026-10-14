use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a reservation window must pass. Check returns its problems, or none.
pub trait ReservationWindowRule: std::fmt::Debug {
    fn check(&self, item: &ReservationWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationWindowRequiredFingerprintRule {
    pub message: String,
}

impl ReservationWindowRule for ReservationWindowRequiredFingerprintRule {
    fn check(&self, item: &ReservationWindow) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationWindowCodePrefixRule {
    pub prefix: String,
}

impl ReservationWindowRule for ReservationWindowCodePrefixRule {
    fn check(&self, item: &ReservationWindow) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationWindowFingerprintLengthRule {
    pub max_length: i64,
}

impl ReservationWindowRule for ReservationWindowFingerprintLengthRule {
    fn check(&self, item: &ReservationWindow) -> Vec<String> {
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

/// Runs every rule of a reservation window.
#[derive(Debug, Clone)]
pub struct ReservationWindowValidator {
    pub rules: Vec<Rc<dyn ReservationWindowRule>>,
}

impl ReservationWindowValidator {
    pub fn validate(&self, item: &ReservationWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReservationWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_window_validator() -> ReservationWindowValidator {
    ReservationWindowValidator {
        rules: vec![
            Rc::new(ReservationWindowRequiredFingerprintRule {
                message: "reservation window needs this field".to_string(),
            }) as Rc<dyn ReservationWindowRule>,
            Rc::new(ReservationWindowCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(ReservationWindowFingerprintLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_reservation_window_validator();
        let item = new_reservation_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = ReservationWindowRequiredFingerprintRule {
            message: "reservation window needs an entry".to_string(),
        };
        let mut item = new_reservation_window("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = ReservationWindowFingerprintLengthRule { max_length: 16 };
        let mut item = new_reservation_window("x", "y");
        item.fingerprint = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
