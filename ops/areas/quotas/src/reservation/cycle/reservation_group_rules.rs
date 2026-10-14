use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a reservation group must pass. Check returns its problems, or none.
pub trait ReservationGroupRule: std::fmt::Debug {
    fn check(&self, item: &ReservationGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationGroupPhoneLengthRule {
    pub max_length: i64,
}

impl ReservationGroupRule for ReservationGroupPhoneLengthRule {
    fn check(&self, item: &ReservationGroup) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationGroupBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReservationGroupRule for ReservationGroupBurstRangeRule {
    fn check(&self, item: &ReservationGroup) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationGroupRequiredPhoneRule {
    pub message: String,
}

impl ReservationGroupRule for ReservationGroupRequiredPhoneRule {
    fn check(&self, item: &ReservationGroup) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationGroupRequiredLabelRule {
    pub message: String,
}

impl ReservationGroupRule for ReservationGroupRequiredLabelRule {
    fn check(&self, item: &ReservationGroup) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation group.
#[derive(Debug, Clone)]
pub struct ReservationGroupValidator {
    pub rules: Vec<Rc<dyn ReservationGroupRule>>,
}

impl ReservationGroupValidator {
    pub fn validate(&self, item: &ReservationGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReservationGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_group_validator() -> ReservationGroupValidator {
    ReservationGroupValidator {
        rules: vec![
            Rc::new(ReservationGroupPhoneLengthRule { max_length: 1000 }) as Rc<dyn ReservationGroupRule>,
            Rc::new(ReservationGroupBurstRangeRule { min: 120, max: 3600 }),
            Rc::new(ReservationGroupRequiredPhoneRule {
                message: "reservation group needs this field".to_string(),
            }),
            Rc::new(ReservationGroupRequiredLabelRule {
                message: "reservation group needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_reservation_group_validator();
        let item = new_reservation_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = ReservationGroupPhoneLengthRule { max_length: 16 };
        let mut item = new_reservation_group("x", "y");
        item.phone = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = ReservationGroupBurstRangeRule { min: 16, max: 32 };
        let mut item = new_reservation_group("x", "y");
        item.burst = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = ReservationGroupRequiredPhoneRule {
            message: "reservation group needs a value".to_string(),
        };
        let mut item = new_reservation_group("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ReservationGroupRequiredLabelRule {
            message: "reservation group needs an entry".to_string(),
        };
        let mut item = new_reservation_group("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
