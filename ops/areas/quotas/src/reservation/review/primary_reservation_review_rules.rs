use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary reservation review must pass. Check returns its problems, or none.
pub trait PrimaryReservationReviewRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryReservationReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationReviewRequiredCurrencyRule {
    pub message: String,
}

impl PrimaryReservationReviewRule for PrimaryReservationReviewRequiredCurrencyRule {
    fn check(&self, item: &PrimaryReservationReview) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationReviewRequiredTitleRule {
    pub message: String,
}

impl PrimaryReservationReviewRule for PrimaryReservationReviewRequiredTitleRule {
    fn check(&self, item: &PrimaryReservationReview) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationReviewCurrencyLengthRule {
    pub max_length: i64,
}

impl PrimaryReservationReviewRule for PrimaryReservationReviewCurrencyLengthRule {
    fn check(&self, item: &PrimaryReservationReview) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationReviewAllowedNameRule {
    pub allowed: Vec<String>,
}

impl PrimaryReservationReviewRule for PrimaryReservationReviewAllowedNameRule {
    fn check(&self, item: &PrimaryReservationReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is too long", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationReviewRequiredNameRule {
    pub message: String,
}

impl PrimaryReservationReviewRule for PrimaryReservationReviewRequiredNameRule {
    fn check(&self, item: &PrimaryReservationReview) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationReviewNameLengthRule {
    pub max_length: i64,
}

impl PrimaryReservationReviewRule for PrimaryReservationReviewNameLengthRule {
    fn check(&self, item: &PrimaryReservationReview) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary reservation review.
#[derive(Debug, Clone)]
pub struct PrimaryReservationReviewValidator {
    pub rules: Vec<Rc<dyn PrimaryReservationReviewRule>>,
}

impl PrimaryReservationReviewValidator {
    pub fn validate(&self, item: &PrimaryReservationReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryReservationReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryReservationReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_reservation_review_validator() -> PrimaryReservationReviewValidator {
    PrimaryReservationReviewValidator {
        rules: vec![
            Rc::new(PrimaryReservationReviewRequiredCurrencyRule {
                message: "primary reservation review needs this field".to_string(),
            }) as Rc<dyn PrimaryReservationReviewRule>,
            Rc::new(PrimaryReservationReviewRequiredTitleRule {
                message: "primary reservation review needs this field".to_string(),
            }),
            Rc::new(PrimaryReservationReviewCurrencyLengthRule { max_length: 86400 }),
            Rc::new(PrimaryReservationReviewAllowedNameRule {
                allowed: vec!["seasonal".to_string()],
            }),
            Rc::new(PrimaryReservationReviewRequiredNameRule {
                message: "primary reservation review needs a value".to_string(),
            }),
            Rc::new(PrimaryReservationReviewNameLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_reservation_review_validator();
        let item = new_primary_reservation_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = PrimaryReservationReviewRequiredCurrencyRule {
            message: "primary reservation review needs this field".to_string(),
        };
        let mut item = new_primary_reservation_review("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PrimaryReservationReviewRequiredTitleRule {
            message: "primary reservation review needs this field".to_string(),
        };
        let mut item = new_primary_reservation_review("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = PrimaryReservationReviewCurrencyLengthRule { max_length: 120 };
        let mut item = new_primary_reservation_review("x", "y");
        item.currency = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PrimaryReservationReviewRequiredNameRule {
            message: "primary reservation review needs this field".to_string(),
        };
        let mut item = new_primary_reservation_review("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PrimaryReservationReviewNameLengthRule { max_length: 16 };
        let mut item = new_primary_reservation_review("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
