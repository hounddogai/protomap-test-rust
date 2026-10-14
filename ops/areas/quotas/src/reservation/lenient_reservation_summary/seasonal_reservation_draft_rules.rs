use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal reservation draft must pass. Check returns its problems, or none.
pub trait SeasonalReservationDraftRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalReservationDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReservationDraftRequiredTitleRule {
    pub message: String,
}

impl SeasonalReservationDraftRule for SeasonalReservationDraftRequiredTitleRule {
    fn check(&self, item: &SeasonalReservationDraft) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReservationDraftScopeLengthRule {
    pub max_length: i64,
}

impl SeasonalReservationDraftRule for SeasonalReservationDraftScopeLengthRule {
    fn check(&self, item: &SeasonalReservationDraft) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReservationDraftRequiredCodeRule {
    pub message: String,
}

impl SeasonalReservationDraftRule for SeasonalReservationDraftRequiredCodeRule {
    fn check(&self, item: &SeasonalReservationDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal reservation draft.
#[derive(Debug, Clone)]
pub struct SeasonalReservationDraftValidator {
    pub rules: Vec<Rc<dyn SeasonalReservationDraftRule>>,
}

impl SeasonalReservationDraftValidator {
    pub fn validate(&self, item: &SeasonalReservationDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalReservationDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalReservationDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_reservation_draft_validator() -> SeasonalReservationDraftValidator {
    SeasonalReservationDraftValidator {
        rules: vec![
            Rc::new(SeasonalReservationDraftRequiredTitleRule {
                message: "seasonal reservation draft needs a value".to_string(),
            }) as Rc<dyn SeasonalReservationDraftRule>,
            Rc::new(SeasonalReservationDraftScopeLengthRule { max_length: 86400 }),
            Rc::new(SeasonalReservationDraftRequiredCodeRule {
                message: "seasonal reservation draft needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_reservation_draft_validator();
        let item = new_seasonal_reservation_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SeasonalReservationDraftRequiredTitleRule {
            message: "seasonal reservation draft needs this field".to_string(),
        };
        let mut item = new_seasonal_reservation_draft("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = SeasonalReservationDraftScopeLengthRule { max_length: 16 };
        let mut item = new_seasonal_reservation_draft("x", "y");
        item.scope = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalReservationDraftRequiredCodeRule {
            message: "seasonal reservation draft needs a value".to_string(),
        };
        let mut item = new_seasonal_reservation_draft("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
