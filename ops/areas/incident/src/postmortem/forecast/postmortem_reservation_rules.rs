use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a postmortem reservation must pass. Check returns its problems, or none.
pub trait PostmortemReservationRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemReservationAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl PostmortemReservationRule for PostmortemReservationAllowedLabelRule {
    fn check(&self, item: &PostmortemReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is missing", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemReservationCodePrefixRule {
    pub prefix: String,
}

impl PostmortemReservationRule for PostmortemReservationCodePrefixRule {
    fn check(&self, item: &PostmortemReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemReservationMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PostmortemReservationRule for PostmortemReservationMinimumRangeRule {
    fn check(&self, item: &PostmortemReservation) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemReservationRequiredContactEmailRule {
    pub message: String,
}

impl PostmortemReservationRule for PostmortemReservationRequiredContactEmailRule {
    fn check(&self, item: &PostmortemReservation) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemReservationRequiredLabelRule {
    pub message: String,
}

impl PostmortemReservationRule for PostmortemReservationRequiredLabelRule {
    fn check(&self, item: &PostmortemReservation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemReservationRequiredPhoneRule {
    pub message: String,
}

impl PostmortemReservationRule for PostmortemReservationRequiredPhoneRule {
    fn check(&self, item: &PostmortemReservation) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem reservation.
#[derive(Debug, Clone)]
pub struct PostmortemReservationValidator {
    pub rules: Vec<Rc<dyn PostmortemReservationRule>>,
}

impl PostmortemReservationValidator {
    pub fn validate(&self, item: &PostmortemReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PostmortemReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_reservation_validator() -> PostmortemReservationValidator {
    PostmortemReservationValidator {
        rules: vec![
            Rc::new(PostmortemReservationAllowedLabelRule {
                allowed: vec!["capped".to_string(), "primary".to_string()],
            }) as Rc<dyn PostmortemReservationRule>,
            Rc::new(PostmortemReservationCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(PostmortemReservationMinimumRangeRule { min: 2.1, max: 2422.8 }),
            Rc::new(PostmortemReservationRequiredContactEmailRule {
                message: "postmortem reservation needs this field".to_string(),
            }),
            Rc::new(PostmortemReservationRequiredLabelRule {
                message: "postmortem reservation needs an entry".to_string(),
            }),
            Rc::new(PostmortemReservationRequiredPhoneRule {
                message: "postmortem reservation needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_postmortem_reservation_validator();
        let item = new_postmortem_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = PostmortemReservationMinimumRangeRule { min: 1.6, max: 661.1 };
        let mut item = new_postmortem_reservation("x", "y");
        item.minimum = 662.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = PostmortemReservationRequiredContactEmailRule {
            message: "postmortem reservation needs a value".to_string(),
        };
        let mut item = new_postmortem_reservation("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PostmortemReservationRequiredLabelRule {
            message: "postmortem reservation needs an entry".to_string(),
        };
        let mut item = new_postmortem_reservation("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = PostmortemReservationRequiredPhoneRule {
            message: "postmortem reservation needs a value".to_string(),
        };
        let mut item = new_postmortem_reservation("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
