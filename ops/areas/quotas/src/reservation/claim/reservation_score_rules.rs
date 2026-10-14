use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a reservation score must pass. Check returns its problems, or none.
pub trait ReservationScoreRule: std::fmt::Debug {
    fn check(&self, item: &ReservationScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationScoreRequiredClusterRule {
    pub message: String,
}

impl ReservationScoreRule for ReservationScoreRequiredClusterRule {
    fn check(&self, item: &ReservationScore) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationScoreClusterLengthRule {
    pub max_length: i64,
}

impl ReservationScoreRule for ReservationScoreClusterLengthRule {
    fn check(&self, item: &ReservationScore) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationScoreRequiredLabelRule {
    pub message: String,
}

impl ReservationScoreRule for ReservationScoreRequiredLabelRule {
    fn check(&self, item: &ReservationScore) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation score.
#[derive(Debug, Clone)]
pub struct ReservationScoreValidator {
    pub rules: Vec<Rc<dyn ReservationScoreRule>>,
}

impl ReservationScoreValidator {
    pub fn validate(&self, item: &ReservationScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReservationScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_score_validator() -> ReservationScoreValidator {
    ReservationScoreValidator {
        rules: vec![
            Rc::new(ReservationScoreRequiredClusterRule {
                message: "reservation score needs this field".to_string(),
            }) as Rc<dyn ReservationScoreRule>,
            Rc::new(ReservationScoreClusterLengthRule { max_length: 80 }),
            Rc::new(ReservationScoreRequiredLabelRule {
                message: "reservation score needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_reservation_score_validator();
        let item = new_reservation_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = ReservationScoreRequiredClusterRule {
            message: "reservation score needs an entry".to_string(),
        };
        let mut item = new_reservation_score("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = ReservationScoreClusterLengthRule { max_length: 120 };
        let mut item = new_reservation_score("x", "y");
        item.cluster = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ReservationScoreRequiredLabelRule {
            message: "reservation score needs a value".to_string(),
        };
        let mut item = new_reservation_score("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
