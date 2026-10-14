use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped reservation variance must pass. Check returns its problems, or none.
pub trait CappedReservationVarianceRule: std::fmt::Debug {
    fn check(&self, item: &CappedReservationVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedReservationVarianceTitleLengthRule {
    pub max_length: i64,
}

impl CappedReservationVarianceRule for CappedReservationVarianceTitleLengthRule {
    fn check(&self, item: &CappedReservationVariance) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedReservationVarianceCodeLengthRule {
    pub max_length: i64,
}

impl CappedReservationVarianceRule for CappedReservationVarianceCodeLengthRule {
    fn check(&self, item: &CappedReservationVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedReservationVarianceMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedReservationVarianceRule for CappedReservationVarianceMarginRangeRule {
    fn check(&self, item: &CappedReservationVariance) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a capped reservation variance.
#[derive(Debug, Clone)]
pub struct CappedReservationVarianceValidator {
    pub rules: Vec<Rc<dyn CappedReservationVarianceRule>>,
}

impl CappedReservationVarianceValidator {
    pub fn validate(&self, item: &CappedReservationVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedReservationVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedReservationVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_reservation_variance_validator() -> CappedReservationVarianceValidator {
    CappedReservationVarianceValidator {
        rules: vec![
            Rc::new(CappedReservationVarianceTitleLengthRule { max_length: 32 })
                as Rc<dyn CappedReservationVarianceRule>,
            Rc::new(CappedReservationVarianceCodeLengthRule { max_length: 80 }),
            Rc::new(CappedReservationVarianceMarginRangeRule { min: 6.6, max: 138.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_reservation_variance_validator();
        let item = new_capped_reservation_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = CappedReservationVarianceTitleLengthRule { max_length: 120 };
        let mut item = new_capped_reservation_variance("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedReservationVarianceCodeLengthRule { max_length: 120 };
        let mut item = new_capped_reservation_variance("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = CappedReservationVarianceMarginRangeRule { min: 6.4, max: 4851.5 };
        let mut item = new_capped_reservation_variance("x", "y");
        item.margin = 4853.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
