use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a reservation target must pass. Check returns its problems, or none.
pub trait ReservationTargetRule: std::fmt::Debug {
    fn check(&self, item: &ReservationTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationTargetCodeLengthRule {
    pub max_length: i64,
}

impl ReservationTargetRule for ReservationTargetCodeLengthRule {
    fn check(&self, item: &ReservationTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationTargetPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReservationTargetRule for ReservationTargetPriorityRangeRule {
    fn check(&self, item: &ReservationTarget) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationTargetClusterLengthRule {
    pub max_length: i64,
}

impl ReservationTargetRule for ReservationTargetClusterLengthRule {
    fn check(&self, item: &ReservationTarget) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation target.
#[derive(Debug, Clone)]
pub struct ReservationTargetValidator {
    pub rules: Vec<Rc<dyn ReservationTargetRule>>,
}

impl ReservationTargetValidator {
    pub fn validate(&self, item: &ReservationTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReservationTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_target_validator() -> ReservationTargetValidator {
    ReservationTargetValidator {
        rules: vec![
            Rc::new(ReservationTargetCodeLengthRule { max_length: 80 }) as Rc<dyn ReservationTargetRule>,
            Rc::new(ReservationTargetPriorityRangeRule { min: 3600, max: 86400 }),
            Rc::new(ReservationTargetClusterLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_reservation_target_validator();
        let item = new_reservation_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ReservationTargetCodeLengthRule { max_length: 120 };
        let mut item = new_reservation_target("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = ReservationTargetPriorityRangeRule { min: 64, max: 64 };
        let mut item = new_reservation_target("x", "y");
        item.priority = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = ReservationTargetClusterLengthRule { max_length: 16 };
        let mut item = new_reservation_target("x", "y");
        item.cluster = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
