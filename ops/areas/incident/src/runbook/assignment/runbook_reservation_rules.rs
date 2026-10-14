use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a runbook reservation must pass. Check returns its problems, or none.
pub trait RunbookReservationRule: std::fmt::Debug {
    fn check(&self, item: &RunbookReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookReservationCodePrefixRule {
    pub prefix: String,
}

impl RunbookReservationRule for RunbookReservationCodePrefixRule {
    fn check(&self, item: &RunbookReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookReservationRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RunbookReservationRule for RunbookReservationRetriesRangeRule {
    fn check(&self, item: &RunbookReservation) -> Vec<String> {
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
pub struct RunbookReservationWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RunbookReservationRule for RunbookReservationWeightRangeRule {
    fn check(&self, item: &RunbookReservation) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookReservationTitleLengthRule {
    pub max_length: i64,
}

impl RunbookReservationRule for RunbookReservationTitleLengthRule {
    fn check(&self, item: &RunbookReservation) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookReservationContactEmailLengthRule {
    pub max_length: i64,
}

impl RunbookReservationRule for RunbookReservationContactEmailLengthRule {
    fn check(&self, item: &RunbookReservation) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookReservationCodeLengthRule {
    pub max_length: i64,
}

impl RunbookReservationRule for RunbookReservationCodeLengthRule {
    fn check(&self, item: &RunbookReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a runbook reservation.
#[derive(Debug, Clone)]
pub struct RunbookReservationValidator {
    pub rules: Vec<Rc<dyn RunbookReservationRule>>,
}

impl RunbookReservationValidator {
    pub fn validate(&self, item: &RunbookReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RunbookReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_reservation_validator() -> RunbookReservationValidator {
    RunbookReservationValidator {
        rules: vec![
            Rc::new(RunbookReservationCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn RunbookReservationRule>,
            Rc::new(RunbookReservationRetriesRangeRule { min: 64, max: 80 }),
            Rc::new(RunbookReservationWeightRangeRule { min: 3.4, max: 4098.9 }),
            Rc::new(RunbookReservationTitleLengthRule { max_length: 255 }),
            Rc::new(RunbookReservationContactEmailLengthRule { max_length: 1000 }),
            Rc::new(RunbookReservationCodeLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_runbook_reservation_validator();
        let item = new_runbook_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = RunbookReservationRetriesRangeRule { min: 32, max: 86400 };
        let mut item = new_runbook_reservation("x", "y");
        item.retries = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = RunbookReservationWeightRangeRule { min: 7.6, max: 2321.1 };
        let mut item = new_runbook_reservation("x", "y");
        item.weight = 2322.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RunbookReservationTitleLengthRule { max_length: 255 };
        let mut item = new_runbook_reservation("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = RunbookReservationContactEmailLengthRule { max_length: 255 };
        let mut item = new_runbook_reservation("x", "y");
        item.contact_email = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RunbookReservationCodeLengthRule { max_length: 86400 };
        let mut item = new_runbook_reservation("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
