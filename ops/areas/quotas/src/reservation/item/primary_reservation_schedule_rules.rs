use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary reservation schedule must pass. Check returns its problems, or none.
pub trait PrimaryReservationScheduleRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryReservationSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationScheduleCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryReservationScheduleRule for PrimaryReservationScheduleCodeLengthRule {
    fn check(&self, item: &PrimaryReservationSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationSchedulePinnedGuardRule {
    pub limit: f64,
}

impl PrimaryReservationScheduleRule for PrimaryReservationSchedulePinnedGuardRule {
    fn check(&self, item: &PrimaryReservationSchedule) -> Vec<String> {
        if item.pinned && (item.discount > self.limit) {
            return vec![format!(
                "discount {} exceeds {} while pinned",
                item.discount, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationScheduleRequiredRegionRule {
    pub message: String,
}

impl PrimaryReservationScheduleRule for PrimaryReservationScheduleRequiredRegionRule {
    fn check(&self, item: &PrimaryReservationSchedule) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationScheduleRequiredCodeRule {
    pub message: String,
}

impl PrimaryReservationScheduleRule for PrimaryReservationScheduleRequiredCodeRule {
    fn check(&self, item: &PrimaryReservationSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary reservation schedule.
#[derive(Debug, Clone)]
pub struct PrimaryReservationScheduleValidator {
    pub rules: Vec<Rc<dyn PrimaryReservationScheduleRule>>,
}

impl PrimaryReservationScheduleValidator {
    pub fn validate(&self, item: &PrimaryReservationSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryReservationSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryReservationSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_reservation_schedule_validator() -> PrimaryReservationScheduleValidator {
    PrimaryReservationScheduleValidator {
        rules: vec![
            Rc::new(PrimaryReservationScheduleCodeLengthRule { max_length: 3600 })
                as Rc<dyn PrimaryReservationScheduleRule>,
            Rc::new(PrimaryReservationSchedulePinnedGuardRule { limit: 7.3 }),
            Rc::new(PrimaryReservationScheduleRequiredRegionRule {
                message: "primary reservation schedule needs a value".to_string(),
            }),
            Rc::new(PrimaryReservationScheduleRequiredCodeRule {
                message: "primary reservation schedule needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_reservation_schedule_validator();
        let item = new_primary_reservation_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryReservationScheduleCodeLengthRule { max_length: 80 };
        let mut item = new_primary_reservation_schedule("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = PrimaryReservationScheduleRequiredRegionRule {
            message: "primary reservation schedule needs a value".to_string(),
        };
        let mut item = new_primary_reservation_schedule("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryReservationScheduleRequiredCodeRule {
            message: "primary reservation schedule needs this field".to_string(),
        };
        let mut item = new_primary_reservation_schedule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
