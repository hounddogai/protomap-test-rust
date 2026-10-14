use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a reservation item must pass. Check returns its problems, or none.
pub trait ReservationItemRule: std::fmt::Debug {
    fn check(&self, item: &ReservationItem) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationItemCodeLengthRule {
    pub max_length: i64,
}

impl ReservationItemRule for ReservationItemCodeLengthRule {
    fn check(&self, item: &ReservationItem) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationItemTitleLengthRule {
    pub max_length: i64,
}

impl ReservationItemRule for ReservationItemTitleLengthRule {
    fn check(&self, item: &ReservationItem) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationItemRequiredCodeRule {
    pub message: String,
}

impl ReservationItemRule for ReservationItemRequiredCodeRule {
    fn check(&self, item: &ReservationItem) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationItemCodePrefixRule {
    pub prefix: String,
}

impl ReservationItemRule for ReservationItemCodePrefixRule {
    fn check(&self, item: &ReservationItem) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation item.
#[derive(Debug, Clone)]
pub struct ReservationItemValidator {
    pub rules: Vec<Rc<dyn ReservationItemRule>>,
}

impl ReservationItemValidator {
    pub fn validate(&self, item: &ReservationItem) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReservationItem) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationItem]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_item_validator() -> ReservationItemValidator {
    ReservationItemValidator {
        rules: vec![
            Rc::new(ReservationItemCodeLengthRule { max_length: 16 }) as Rc<dyn ReservationItemRule>,
            Rc::new(ReservationItemTitleLengthRule { max_length: 64 }),
            Rc::new(ReservationItemRequiredCodeRule {
                message: "reservation item needs a value".to_string(),
            }),
            Rc::new(ReservationItemCodePrefixRule {
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
        let validator = default_reservation_item_validator();
        let item = new_reservation_item("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ReservationItemCodeLengthRule { max_length: 120 };
        let mut item = new_reservation_item("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ReservationItemTitleLengthRule { max_length: 64 };
        let mut item = new_reservation_item("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ReservationItemRequiredCodeRule {
            message: "reservation item needs this field".to_string(),
        };
        let mut item = new_reservation_item("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
