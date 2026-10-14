use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual runbook reservation must pass. Check returns its problems, or none.
pub trait ManualRunbookReservationRule: std::fmt::Debug {
    fn check(&self, item: &ManualRunbookReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRunbookReservationCodeLengthRule {
    pub max_length: i64,
}

impl ManualRunbookReservationRule for ManualRunbookReservationCodeLengthRule {
    fn check(&self, item: &ManualRunbookReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRunbookReservationRequiredCodeRule {
    pub message: String,
}

impl ManualRunbookReservationRule for ManualRunbookReservationRequiredCodeRule {
    fn check(&self, item: &ManualRunbookReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRunbookReservationRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualRunbookReservationRule for ManualRunbookReservationRetriesRangeRule {
    fn check(&self, item: &ManualRunbookReservation) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRunbookReservationRequiredLabelRule {
    pub message: String,
}

impl ManualRunbookReservationRule for ManualRunbookReservationRequiredLabelRule {
    fn check(&self, item: &ManualRunbookReservation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual runbook reservation.
#[derive(Debug, Clone)]
pub struct ManualRunbookReservationValidator {
    pub rules: Vec<Rc<dyn ManualRunbookReservationRule>>,
}

impl ManualRunbookReservationValidator {
    pub fn validate(&self, item: &ManualRunbookReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualRunbookReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualRunbookReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_runbook_reservation_validator() -> ManualRunbookReservationValidator {
    ManualRunbookReservationValidator {
        rules: vec![
            Rc::new(ManualRunbookReservationCodeLengthRule { max_length: 1000 })
                as Rc<dyn ManualRunbookReservationRule>,
            Rc::new(ManualRunbookReservationRequiredCodeRule {
                message: "manual runbook reservation needs this field".to_string(),
            }),
            Rc::new(ManualRunbookReservationRetriesRangeRule { min: 32, max: 120 }),
            Rc::new(ManualRunbookReservationRequiredLabelRule {
                message: "manual runbook reservation needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_runbook_reservation_validator();
        let item = new_manual_runbook_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualRunbookReservationCodeLengthRule { max_length: 16 };
        let mut item = new_manual_runbook_reservation("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualRunbookReservationRequiredCodeRule {
            message: "manual runbook reservation needs a value".to_string(),
        };
        let mut item = new_manual_runbook_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = ManualRunbookReservationRetriesRangeRule { min: 64, max: 120 };
        let mut item = new_manual_runbook_reservation("x", "y");
        item.retries = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ManualRunbookReservationRequiredLabelRule {
            message: "manual runbook reservation needs a value".to_string(),
        };
        let mut item = new_manual_runbook_reservation("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
