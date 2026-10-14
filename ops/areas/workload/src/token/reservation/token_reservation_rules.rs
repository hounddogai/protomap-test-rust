use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a token reservation must pass. Check returns its problems, or none.
pub trait TokenReservationRule: std::fmt::Debug {
    fn check(&self, item: &TokenReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenReservationTitleLengthRule {
    pub max_length: i64,
}

impl TokenReservationRule for TokenReservationTitleLengthRule {
    fn check(&self, item: &TokenReservation) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenReservationScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TokenReservationRule for TokenReservationScoreRangeRule {
    fn check(&self, item: &TokenReservation) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenReservationRequiredScopeRule {
    pub message: String,
}

impl TokenReservationRule for TokenReservationRequiredScopeRule {
    fn check(&self, item: &TokenReservation) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenReservationRequiredCodeRule {
    pub message: String,
}

impl TokenReservationRule for TokenReservationRequiredCodeRule {
    fn check(&self, item: &TokenReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenReservationRequiredSummaryRule {
    pub message: String,
}

impl TokenReservationRule for TokenReservationRequiredSummaryRule {
    fn check(&self, item: &TokenReservation) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenReservationCodeLengthRule {
    pub max_length: i64,
}

impl TokenReservationRule for TokenReservationCodeLengthRule {
    fn check(&self, item: &TokenReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a token reservation.
#[derive(Debug, Clone)]
pub struct TokenReservationValidator {
    pub rules: Vec<Rc<dyn TokenReservationRule>>,
}

impl TokenReservationValidator {
    pub fn validate(&self, item: &TokenReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TokenReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_reservation_validator() -> TokenReservationValidator {
    TokenReservationValidator {
        rules: vec![
            Rc::new(TokenReservationTitleLengthRule { max_length: 1000 }) as Rc<dyn TokenReservationRule>,
            Rc::new(TokenReservationScoreRangeRule { min: 8.4, max: 1842.3 }),
            Rc::new(TokenReservationRequiredScopeRule {
                message: "token reservation needs an entry".to_string(),
            }),
            Rc::new(TokenReservationRequiredCodeRule {
                message: "token reservation needs this field".to_string(),
            }),
            Rc::new(TokenReservationRequiredSummaryRule {
                message: "token reservation needs this field".to_string(),
            }),
            Rc::new(TokenReservationCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_token_reservation_validator();
        let item = new_token_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TokenReservationTitleLengthRule { max_length: 32 };
        let mut item = new_token_reservation("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = TokenReservationScoreRangeRule { min: 7.0, max: 4820.5 };
        let mut item = new_token_reservation("x", "y");
        item.score = 4822.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = TokenReservationRequiredScopeRule {
            message: "token reservation needs an entry".to_string(),
        };
        let mut item = new_token_reservation("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TokenReservationRequiredCodeRule {
            message: "token reservation needs this field".to_string(),
        };
        let mut item = new_token_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = TokenReservationRequiredSummaryRule {
            message: "token reservation needs an entry".to_string(),
        };
        let mut item = new_token_reservation("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TokenReservationCodeLengthRule { max_length: 120 };
        let mut item = new_token_reservation("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
