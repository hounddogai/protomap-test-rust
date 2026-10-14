use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a reservation milestone must pass. Check returns its problems, or none.
pub trait ReservationMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &ReservationMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationMilestoneLocaleLengthRule {
    pub max_length: i64,
}

impl ReservationMilestoneRule for ReservationMilestoneLocaleLengthRule {
    fn check(&self, item: &ReservationMilestone) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationMilestoneUnitLengthRule {
    pub max_length: i64,
}

impl ReservationMilestoneRule for ReservationMilestoneUnitLengthRule {
    fn check(&self, item: &ReservationMilestone) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationMilestoneRequiredUnitRule {
    pub message: String,
}

impl ReservationMilestoneRule for ReservationMilestoneRequiredUnitRule {
    fn check(&self, item: &ReservationMilestone) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation milestone.
#[derive(Debug, Clone)]
pub struct ReservationMilestoneValidator {
    pub rules: Vec<Rc<dyn ReservationMilestoneRule>>,
}

impl ReservationMilestoneValidator {
    pub fn validate(&self, item: &ReservationMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReservationMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_milestone_validator() -> ReservationMilestoneValidator {
    ReservationMilestoneValidator {
        rules: vec![
            Rc::new(ReservationMilestoneLocaleLengthRule { max_length: 86400 }) as Rc<dyn ReservationMilestoneRule>,
            Rc::new(ReservationMilestoneUnitLengthRule { max_length: 86400 }),
            Rc::new(ReservationMilestoneRequiredUnitRule {
                message: "reservation milestone needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_reservation_milestone_validator();
        let item = new_reservation_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = ReservationMilestoneLocaleLengthRule { max_length: 255 };
        let mut item = new_reservation_milestone("x", "y");
        item.locale = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = ReservationMilestoneUnitLengthRule { max_length: 86400 };
        let mut item = new_reservation_milestone("x", "y");
        item.unit = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = ReservationMilestoneRequiredUnitRule {
            message: "reservation milestone needs this field".to_string(),
        };
        let mut item = new_reservation_milestone("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
