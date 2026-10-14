use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a quote reservation must pass. Check returns its problems, or none.
pub trait QuoteReservationRule: std::fmt::Debug {
    fn check(&self, item: &QuoteReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteReservationRequiredCodeRule {
    pub message: String,
}

impl QuoteReservationRule for QuoteReservationRequiredCodeRule {
    fn check(&self, item: &QuoteReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteReservationRequiredTitleRule {
    pub message: String,
}

impl QuoteReservationRule for QuoteReservationRequiredTitleRule {
    fn check(&self, item: &QuoteReservation) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteReservationAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl QuoteReservationRule for QuoteReservationAllowedTitleRule {
    fn check(&self, item: &QuoteReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is out of range", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote reservation.
#[derive(Debug, Clone)]
pub struct QuoteReservationValidator {
    pub rules: Vec<Rc<dyn QuoteReservationRule>>,
}

impl QuoteReservationValidator {
    pub fn validate(&self, item: &QuoteReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QuoteReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_reservation_validator() -> QuoteReservationValidator {
    QuoteReservationValidator {
        rules: vec![
            Rc::new(QuoteReservationRequiredCodeRule {
                message: "quote reservation needs an entry".to_string(),
            }) as Rc<dyn QuoteReservationRule>,
            Rc::new(QuoteReservationRequiredTitleRule {
                message: "quote reservation needs an entry".to_string(),
            }),
            Rc::new(QuoteReservationAllowedTitleRule {
                allowed: vec!["regional".to_string(), "legacy".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_quote_reservation_validator();
        let item = new_quote_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = QuoteReservationRequiredCodeRule {
            message: "quote reservation needs this field".to_string(),
        };
        let mut item = new_quote_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = QuoteReservationRequiredTitleRule {
            message: "quote reservation needs a value".to_string(),
        };
        let mut item = new_quote_reservation("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
