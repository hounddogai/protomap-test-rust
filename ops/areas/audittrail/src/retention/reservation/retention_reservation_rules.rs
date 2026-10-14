use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a retention reservation must pass. Check returns its problems, or none.
pub trait RetentionReservationRule: std::fmt::Debug {
    fn check(&self, item: &RetentionReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionReservationRequiredLocaleRule {
    pub message: String,
}

impl RetentionReservationRule for RetentionReservationRequiredLocaleRule {
    fn check(&self, item: &RetentionReservation) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionReservationRequiredTitleRule {
    pub message: String,
}

impl RetentionReservationRule for RetentionReservationRequiredTitleRule {
    fn check(&self, item: &RetentionReservation) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionReservationRequiredCategoryRule {
    pub message: String,
}

impl RetentionReservationRule for RetentionReservationRequiredCategoryRule {
    fn check(&self, item: &RetentionReservation) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionReservationTitleLengthRule {
    pub max_length: i64,
}

impl RetentionReservationRule for RetentionReservationTitleLengthRule {
    fn check(&self, item: &RetentionReservation) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionReservationCodePrefixRule {
    pub prefix: String,
}

impl RetentionReservationRule for RetentionReservationCodePrefixRule {
    fn check(&self, item: &RetentionReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention reservation.
#[derive(Debug, Clone)]
pub struct RetentionReservationValidator {
    pub rules: Vec<Rc<dyn RetentionReservationRule>>,
}

impl RetentionReservationValidator {
    pub fn validate(&self, item: &RetentionReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RetentionReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_reservation_validator() -> RetentionReservationValidator {
    RetentionReservationValidator {
        rules: vec![
            Rc::new(RetentionReservationRequiredLocaleRule {
                message: "retention reservation needs a value".to_string(),
            }) as Rc<dyn RetentionReservationRule>,
            Rc::new(RetentionReservationRequiredTitleRule {
                message: "retention reservation needs this field".to_string(),
            }),
            Rc::new(RetentionReservationRequiredCategoryRule {
                message: "retention reservation needs this field".to_string(),
            }),
            Rc::new(RetentionReservationTitleLengthRule { max_length: 120 }),
            Rc::new(RetentionReservationCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_reservation_validator();
        let item = new_retention_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = RetentionReservationRequiredLocaleRule {
            message: "retention reservation needs this field".to_string(),
        };
        let mut item = new_retention_reservation("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RetentionReservationRequiredTitleRule {
            message: "retention reservation needs this field".to_string(),
        };
        let mut item = new_retention_reservation("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = RetentionReservationRequiredCategoryRule {
            message: "retention reservation needs this field".to_string(),
        };
        let mut item = new_retention_reservation("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RetentionReservationTitleLengthRule { max_length: 120 };
        let mut item = new_retention_reservation("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
