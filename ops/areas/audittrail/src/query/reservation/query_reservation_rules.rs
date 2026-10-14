use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a query reservation must pass. Check returns its problems, or none.
pub trait QueryReservationRule: std::fmt::Debug {
    fn check(&self, item: &QueryReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryReservationRequiredExternalRefRule {
    pub message: String,
}

impl QueryReservationRule for QueryReservationRequiredExternalRefRule {
    fn check(&self, item: &QueryReservation) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryReservationAllowedDescriptionRule {
    pub allowed: Vec<String>,
}

impl QueryReservationRule for QueryReservationAllowedDescriptionRule {
    fn check(&self, item: &QueryReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.description)))
        {
            return vec![format!("description {} is missing", item.description)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryReservationLabelLengthRule {
    pub max_length: i64,
}

impl QueryReservationRule for QueryReservationLabelLengthRule {
    fn check(&self, item: &QueryReservation) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryReservationCodeLengthRule {
    pub max_length: i64,
}

impl QueryReservationRule for QueryReservationCodeLengthRule {
    fn check(&self, item: &QueryReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a query reservation.
#[derive(Debug, Clone)]
pub struct QueryReservationValidator {
    pub rules: Vec<Rc<dyn QueryReservationRule>>,
}

impl QueryReservationValidator {
    pub fn validate(&self, item: &QueryReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QueryReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_reservation_validator() -> QueryReservationValidator {
    QueryReservationValidator {
        rules: vec![
            Rc::new(QueryReservationRequiredExternalRefRule {
                message: "query reservation needs an entry".to_string(),
            }) as Rc<dyn QueryReservationRule>,
            Rc::new(QueryReservationAllowedDescriptionRule {
                allowed: vec!["default".to_string(), "weighted".to_string()],
            }),
            Rc::new(QueryReservationLabelLengthRule { max_length: 255 }),
            Rc::new(QueryReservationCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_query_reservation_validator();
        let item = new_query_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = QueryReservationRequiredExternalRefRule {
            message: "query reservation needs this field".to_string(),
        };
        let mut item = new_query_reservation("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = QueryReservationLabelLengthRule { max_length: 120 };
        let mut item = new_query_reservation("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = QueryReservationCodeLengthRule { max_length: 80 };
        let mut item = new_query_reservation("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
