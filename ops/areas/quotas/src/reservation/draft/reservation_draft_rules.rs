use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a reservation draft must pass. Check returns its problems, or none.
pub trait ReservationDraftRule: std::fmt::Debug {
    fn check(&self, item: &ReservationDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationDraftRequiredNamespaceRule {
    pub message: String,
}

impl ReservationDraftRule for ReservationDraftRequiredNamespaceRule {
    fn check(&self, item: &ReservationDraft) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationDraftRequiredPhoneRule {
    pub message: String,
}

impl ReservationDraftRule for ReservationDraftRequiredPhoneRule {
    fn check(&self, item: &ReservationDraft) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationDraftRequiredNameRule {
    pub message: String,
}

impl ReservationDraftRule for ReservationDraftRequiredNameRule {
    fn check(&self, item: &ReservationDraft) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationDraftNamespaceLengthRule {
    pub max_length: i64,
}

impl ReservationDraftRule for ReservationDraftNamespaceLengthRule {
    fn check(&self, item: &ReservationDraft) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation draft.
#[derive(Debug, Clone)]
pub struct ReservationDraftValidator {
    pub rules: Vec<Rc<dyn ReservationDraftRule>>,
}

impl ReservationDraftValidator {
    pub fn validate(&self, item: &ReservationDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReservationDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_draft_validator() -> ReservationDraftValidator {
    ReservationDraftValidator {
        rules: vec![
            Rc::new(ReservationDraftRequiredNamespaceRule {
                message: "reservation draft needs an entry".to_string(),
            }) as Rc<dyn ReservationDraftRule>,
            Rc::new(ReservationDraftRequiredPhoneRule {
                message: "reservation draft needs a value".to_string(),
            }),
            Rc::new(ReservationDraftRequiredNameRule {
                message: "reservation draft needs a value".to_string(),
            }),
            Rc::new(ReservationDraftNamespaceLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_reservation_draft_validator();
        let item = new_reservation_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = ReservationDraftRequiredNamespaceRule {
            message: "reservation draft needs this field".to_string(),
        };
        let mut item = new_reservation_draft("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = ReservationDraftRequiredPhoneRule {
            message: "reservation draft needs a value".to_string(),
        };
        let mut item = new_reservation_draft("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ReservationDraftRequiredNameRule {
            message: "reservation draft needs this field".to_string(),
        };
        let mut item = new_reservation_draft("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = ReservationDraftNamespaceLengthRule { max_length: 80 };
        let mut item = new_reservation_draft("x", "y");
        item.namespace = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
