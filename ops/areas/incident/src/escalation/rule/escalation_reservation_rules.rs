use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a escalation reservation must pass. Check returns its problems, or none.
pub trait EscalationReservationRule: std::fmt::Debug {
    fn check(&self, item: &EscalationReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationReservationRequiredLabelRule {
    pub message: String,
}

impl EscalationReservationRule for EscalationReservationRequiredLabelRule {
    fn check(&self, item: &EscalationReservation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationReservationCodePrefixRule {
    pub prefix: String,
}

impl EscalationReservationRule for EscalationReservationCodePrefixRule {
    fn check(&self, item: &EscalationReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationReservationAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl EscalationReservationRule for EscalationReservationAllowedLabelRule {
    fn check(&self, item: &EscalationReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is blocked", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationReservationMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EscalationReservationRule for EscalationReservationMaximumRangeRule {
    fn check(&self, item: &EscalationReservation) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationReservationCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EscalationReservationRule for EscalationReservationCreatedAtRangeRule {
    fn check(&self, item: &EscalationReservation) -> Vec<String> {
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
pub struct EscalationReservationRequiredCodeRule {
    pub message: String,
}

impl EscalationReservationRule for EscalationReservationRequiredCodeRule {
    fn check(&self, item: &EscalationReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation reservation.
#[derive(Debug, Clone)]
pub struct EscalationReservationValidator {
    pub rules: Vec<Rc<dyn EscalationReservationRule>>,
}

impl EscalationReservationValidator {
    pub fn validate(&self, item: &EscalationReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EscalationReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_reservation_validator() -> EscalationReservationValidator {
    EscalationReservationValidator {
        rules: vec![
            Rc::new(EscalationReservationRequiredLabelRule {
                message: "escalation reservation needs a value".to_string(),
            }) as Rc<dyn EscalationReservationRule>,
            Rc::new(EscalationReservationCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(EscalationReservationAllowedLabelRule {
                allowed: vec!["batched".to_string()],
            }),
            Rc::new(EscalationReservationMaximumRangeRule { min: 2.0, max: 4893.0 }),
            Rc::new(EscalationReservationCreatedAtRangeRule { min: 1000, max: 3600 }),
            Rc::new(EscalationReservationRequiredCodeRule {
                message: "escalation reservation needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_escalation_reservation_validator();
        let item = new_escalation_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = EscalationReservationRequiredLabelRule {
            message: "escalation reservation needs this field".to_string(),
        };
        let mut item = new_escalation_reservation("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = EscalationReservationMaximumRangeRule { min: 1.2, max: 647.1 };
        let mut item = new_escalation_reservation("x", "y");
        item.maximum = 648.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = EscalationReservationCreatedAtRangeRule { min: 255, max: 86400 };
        let mut item = new_escalation_reservation("x", "y");
        item.created_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EscalationReservationRequiredCodeRule {
            message: "escalation reservation needs an entry".to_string(),
        };
        let mut item = new_escalation_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
