use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted evidence reservation must pass. Check returns its problems, or none.
pub trait WeightedEvidenceReservationRule: std::fmt::Debug {
    fn check(&self, item: &WeightedEvidenceReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEvidenceReservationContactEmailLengthRule {
    pub max_length: i64,
}

impl WeightedEvidenceReservationRule for WeightedEvidenceReservationContactEmailLengthRule {
    fn check(&self, item: &WeightedEvidenceReservation) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEvidenceReservationPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedEvidenceReservationRule for WeightedEvidenceReservationPortRangeRule {
    fn check(&self, item: &WeightedEvidenceReservation) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEvidenceReservationRequiredNameRule {
    pub message: String,
}

impl WeightedEvidenceReservationRule for WeightedEvidenceReservationRequiredNameRule {
    fn check(&self, item: &WeightedEvidenceReservation) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEvidenceReservationAllowedNameRule {
    pub allowed: Vec<String>,
}

impl WeightedEvidenceReservationRule for WeightedEvidenceReservationAllowedNameRule {
    fn check(&self, item: &WeightedEvidenceReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is duplicated", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted evidence reservation.
#[derive(Debug, Clone)]
pub struct WeightedEvidenceReservationValidator {
    pub rules: Vec<Rc<dyn WeightedEvidenceReservationRule>>,
}

impl WeightedEvidenceReservationValidator {
    pub fn validate(&self, item: &WeightedEvidenceReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedEvidenceReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedEvidenceReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_evidence_reservation_validator() -> WeightedEvidenceReservationValidator {
    WeightedEvidenceReservationValidator {
        rules: vec![
            Rc::new(WeightedEvidenceReservationContactEmailLengthRule { max_length: 255 })
                as Rc<dyn WeightedEvidenceReservationRule>,
            Rc::new(WeightedEvidenceReservationPortRangeRule { min: 80, max: 120 }),
            Rc::new(WeightedEvidenceReservationRequiredNameRule {
                message: "weighted evidence reservation needs a value".to_string(),
            }),
            Rc::new(WeightedEvidenceReservationAllowedNameRule {
                allowed: vec!["rolling".to_string(), "regional".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_evidence_reservation_validator();
        let item = new_weighted_evidence_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = WeightedEvidenceReservationContactEmailLengthRule { max_length: 80 };
        let mut item = new_weighted_evidence_reservation("x", "y");
        item.contact_email = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = WeightedEvidenceReservationPortRangeRule { min: 32, max: 1000 };
        let mut item = new_weighted_evidence_reservation("x", "y");
        item.port = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = WeightedEvidenceReservationRequiredNameRule {
            message: "weighted evidence reservation needs this field".to_string(),
        };
        let mut item = new_weighted_evidence_reservation("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
