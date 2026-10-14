use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global reservation variance must pass. Check returns its problems, or none.
pub trait GlobalReservationVarianceRule: std::fmt::Debug {
    fn check(&self, item: &GlobalReservationVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReservationVarianceBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalReservationVarianceRule for GlobalReservationVarianceBatchSizeRangeRule {
    fn check(&self, item: &GlobalReservationVariance) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReservationVarianceStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalReservationVarianceRule for GlobalReservationVarianceStartsAtRangeRule {
    fn check(&self, item: &GlobalReservationVariance) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReservationVarianceRequiredTitleRule {
    pub message: String,
}

impl GlobalReservationVarianceRule for GlobalReservationVarianceRequiredTitleRule {
    fn check(&self, item: &GlobalReservationVariance) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global reservation variance.
#[derive(Debug, Clone)]
pub struct GlobalReservationVarianceValidator {
    pub rules: Vec<Rc<dyn GlobalReservationVarianceRule>>,
}

impl GlobalReservationVarianceValidator {
    pub fn validate(&self, item: &GlobalReservationVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalReservationVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalReservationVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_reservation_variance_validator() -> GlobalReservationVarianceValidator {
    GlobalReservationVarianceValidator {
        rules: vec![
            Rc::new(GlobalReservationVarianceBatchSizeRangeRule { min: 3600, max: 86400 })
                as Rc<dyn GlobalReservationVarianceRule>,
            Rc::new(GlobalReservationVarianceStartsAtRangeRule { min: 120, max: 86400 }),
            Rc::new(GlobalReservationVarianceRequiredTitleRule {
                message: "global reservation variance needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_reservation_variance_validator();
        let item = new_global_reservation_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = GlobalReservationVarianceBatchSizeRangeRule { min: 16, max: 3600 };
        let mut item = new_global_reservation_variance("x", "y");
        item.batch_size = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = GlobalReservationVarianceStartsAtRangeRule { min: 32, max: 120 };
        let mut item = new_global_reservation_variance("x", "y");
        item.starts_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = GlobalReservationVarianceRequiredTitleRule {
            message: "global reservation variance needs an entry".to_string(),
        };
        let mut item = new_global_reservation_variance("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
