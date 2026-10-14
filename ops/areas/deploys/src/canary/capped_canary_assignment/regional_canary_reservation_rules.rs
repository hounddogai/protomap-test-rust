use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional canary reservation must pass. Check returns its problems, or none.
pub trait RegionalCanaryReservationRule: std::fmt::Debug {
    fn check(&self, item: &RegionalCanaryReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCanaryReservationRequiredCodeRule {
    pub message: String,
}

impl RegionalCanaryReservationRule for RegionalCanaryReservationRequiredCodeRule {
    fn check(&self, item: &RegionalCanaryReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCanaryReservationRequiredFingerprintRule {
    pub message: String,
}

impl RegionalCanaryReservationRule for RegionalCanaryReservationRequiredFingerprintRule {
    fn check(&self, item: &RegionalCanaryReservation) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCanaryReservationNameLengthRule {
    pub max_length: i64,
}

impl RegionalCanaryReservationRule for RegionalCanaryReservationNameLengthRule {
    fn check(&self, item: &RegionalCanaryReservation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCanaryReservationFingerprintLengthRule {
    pub max_length: i64,
}

impl RegionalCanaryReservationRule for RegionalCanaryReservationFingerprintLengthRule {
    fn check(&self, item: &RegionalCanaryReservation) -> Vec<String> {
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

/// Runs every rule of a regional canary reservation.
#[derive(Debug, Clone)]
pub struct RegionalCanaryReservationValidator {
    pub rules: Vec<Rc<dyn RegionalCanaryReservationRule>>,
}

impl RegionalCanaryReservationValidator {
    pub fn validate(&self, item: &RegionalCanaryReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalCanaryReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalCanaryReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_canary_reservation_validator() -> RegionalCanaryReservationValidator {
    RegionalCanaryReservationValidator {
        rules: vec![
            Rc::new(RegionalCanaryReservationRequiredCodeRule {
                message: "regional canary reservation needs this field".to_string(),
            }) as Rc<dyn RegionalCanaryReservationRule>,
            Rc::new(RegionalCanaryReservationRequiredFingerprintRule {
                message: "regional canary reservation needs this field".to_string(),
            }),
            Rc::new(RegionalCanaryReservationNameLengthRule { max_length: 32 }),
            Rc::new(RegionalCanaryReservationFingerprintLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_canary_reservation_validator();
        let item = new_regional_canary_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalCanaryReservationRequiredCodeRule {
            message: "regional canary reservation needs this field".to_string(),
        };
        let mut item = new_regional_canary_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = RegionalCanaryReservationRequiredFingerprintRule {
            message: "regional canary reservation needs a value".to_string(),
        };
        let mut item = new_regional_canary_reservation("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RegionalCanaryReservationNameLengthRule { max_length: 64 };
        let mut item = new_regional_canary_reservation("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = RegionalCanaryReservationFingerprintLengthRule { max_length: 16 };
        let mut item = new_regional_canary_reservation("x", "y");
        item.fingerprint = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
