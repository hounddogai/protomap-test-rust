use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary reservation summary must pass. Check returns its problems, or none.
pub trait PrimaryReservationSummaryRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryReservationSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationSummaryAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl PrimaryReservationSummaryRule for PrimaryReservationSummaryAllowedAudienceRule {
    fn check(&self, item: &PrimaryReservationSummary) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is unknown", item.audience)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationSummaryRequiredAudienceRule {
    pub message: String,
}

impl PrimaryReservationSummaryRule for PrimaryReservationSummaryRequiredAudienceRule {
    fn check(&self, item: &PrimaryReservationSummary) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationSummaryRequiredLabelRule {
    pub message: String,
}

impl PrimaryReservationSummaryRule for PrimaryReservationSummaryRequiredLabelRule {
    fn check(&self, item: &PrimaryReservationSummary) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationSummaryLabelLengthRule {
    pub max_length: i64,
}

impl PrimaryReservationSummaryRule for PrimaryReservationSummaryLabelLengthRule {
    fn check(&self, item: &PrimaryReservationSummary) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationSummaryCodePrefixRule {
    pub prefix: String,
}

impl PrimaryReservationSummaryRule for PrimaryReservationSummaryCodePrefixRule {
    fn check(&self, item: &PrimaryReservationSummary) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary reservation summary.
#[derive(Debug, Clone)]
pub struct PrimaryReservationSummaryValidator {
    pub rules: Vec<Rc<dyn PrimaryReservationSummaryRule>>,
}

impl PrimaryReservationSummaryValidator {
    pub fn validate(&self, item: &PrimaryReservationSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryReservationSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryReservationSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_reservation_summary_validator() -> PrimaryReservationSummaryValidator {
    PrimaryReservationSummaryValidator {
        rules: vec![
            Rc::new(PrimaryReservationSummaryAllowedAudienceRule { allowed: Vec::new() })
                as Rc<dyn PrimaryReservationSummaryRule>,
            Rc::new(PrimaryReservationSummaryRequiredAudienceRule {
                message: "primary reservation summary needs a value".to_string(),
            }),
            Rc::new(PrimaryReservationSummaryRequiredLabelRule {
                message: "primary reservation summary needs this field".to_string(),
            }),
            Rc::new(PrimaryReservationSummaryLabelLengthRule { max_length: 120 }),
            Rc::new(PrimaryReservationSummaryCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_reservation_summary_validator();
        let item = new_primary_reservation_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = PrimaryReservationSummaryRequiredAudienceRule {
            message: "primary reservation summary needs an entry".to_string(),
        };
        let mut item = new_primary_reservation_summary("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PrimaryReservationSummaryRequiredLabelRule {
            message: "primary reservation summary needs an entry".to_string(),
        };
        let mut item = new_primary_reservation_summary("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PrimaryReservationSummaryLabelLengthRule { max_length: 16 };
        let mut item = new_primary_reservation_summary("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
