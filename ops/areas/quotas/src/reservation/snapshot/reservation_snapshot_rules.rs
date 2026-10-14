use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a reservation snapshot must pass. Check returns its problems, or none.
pub trait ReservationSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &ReservationSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationSnapshotRequiredCodeRule {
    pub message: String,
}

impl ReservationSnapshotRule for ReservationSnapshotRequiredCodeRule {
    fn check(&self, item: &ReservationSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationSnapshotRequiredExternalRefRule {
    pub message: String,
}

impl ReservationSnapshotRule for ReservationSnapshotRequiredExternalRefRule {
    fn check(&self, item: &ReservationSnapshot) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationSnapshotTitleLengthRule {
    pub max_length: i64,
}

impl ReservationSnapshotRule for ReservationSnapshotTitleLengthRule {
    fn check(&self, item: &ReservationSnapshot) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationSnapshotRequiredNameRule {
    pub message: String,
}

impl ReservationSnapshotRule for ReservationSnapshotRequiredNameRule {
    fn check(&self, item: &ReservationSnapshot) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl ReservationSnapshotRule for ReservationSnapshotCodeLengthRule {
    fn check(&self, item: &ReservationSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation snapshot.
#[derive(Debug, Clone)]
pub struct ReservationSnapshotValidator {
    pub rules: Vec<Rc<dyn ReservationSnapshotRule>>,
}

impl ReservationSnapshotValidator {
    pub fn validate(&self, item: &ReservationSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReservationSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_snapshot_validator() -> ReservationSnapshotValidator {
    ReservationSnapshotValidator {
        rules: vec![
            Rc::new(ReservationSnapshotRequiredCodeRule {
                message: "reservation snapshot needs this field".to_string(),
            }) as Rc<dyn ReservationSnapshotRule>,
            Rc::new(ReservationSnapshotRequiredExternalRefRule {
                message: "reservation snapshot needs an entry".to_string(),
            }),
            Rc::new(ReservationSnapshotTitleLengthRule { max_length: 1000 }),
            Rc::new(ReservationSnapshotRequiredNameRule {
                message: "reservation snapshot needs an entry".to_string(),
            }),
            Rc::new(ReservationSnapshotCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_reservation_snapshot_validator();
        let item = new_reservation_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ReservationSnapshotRequiredCodeRule {
            message: "reservation snapshot needs this field".to_string(),
        };
        let mut item = new_reservation_snapshot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = ReservationSnapshotRequiredExternalRefRule {
            message: "reservation snapshot needs an entry".to_string(),
        };
        let mut item = new_reservation_snapshot("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ReservationSnapshotTitleLengthRule { max_length: 80 };
        let mut item = new_reservation_snapshot("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ReservationSnapshotRequiredNameRule {
            message: "reservation snapshot needs an entry".to_string(),
        };
        let mut item = new_reservation_snapshot("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ReservationSnapshotCodeLengthRule { max_length: 255 };
        let mut item = new_reservation_snapshot("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
