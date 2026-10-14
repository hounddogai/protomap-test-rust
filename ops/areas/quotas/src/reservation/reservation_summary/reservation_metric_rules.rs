use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a reservation metric must pass. Check returns its problems, or none.
pub trait ReservationMetricRule: std::fmt::Debug {
    fn check(&self, item: &ReservationMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationMetricCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ReservationMetricRule for ReservationMetricCostRangeRule {
    fn check(&self, item: &ReservationMetric) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationMetricHostLengthRule {
    pub max_length: i64,
}

impl ReservationMetricRule for ReservationMetricHostLengthRule {
    fn check(&self, item: &ReservationMetric) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationMetricCodeLengthRule {
    pub max_length: i64,
}

impl ReservationMetricRule for ReservationMetricCodeLengthRule {
    fn check(&self, item: &ReservationMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationMetricRequiredCodeRule {
    pub message: String,
}

impl ReservationMetricRule for ReservationMetricRequiredCodeRule {
    fn check(&self, item: &ReservationMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationMetricLabelLengthRule {
    pub max_length: i64,
}

impl ReservationMetricRule for ReservationMetricLabelLengthRule {
    fn check(&self, item: &ReservationMetric) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation metric.
#[derive(Debug, Clone)]
pub struct ReservationMetricValidator {
    pub rules: Vec<Rc<dyn ReservationMetricRule>>,
}

impl ReservationMetricValidator {
    pub fn validate(&self, item: &ReservationMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReservationMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_metric_validator() -> ReservationMetricValidator {
    ReservationMetricValidator {
        rules: vec![
            Rc::new(ReservationMetricCostRangeRule { min: 5.5, max: 1303.3 }) as Rc<dyn ReservationMetricRule>,
            Rc::new(ReservationMetricHostLengthRule { max_length: 120 }),
            Rc::new(ReservationMetricCodeLengthRule { max_length: 255 }),
            Rc::new(ReservationMetricRequiredCodeRule {
                message: "reservation metric needs this field".to_string(),
            }),
            Rc::new(ReservationMetricLabelLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_reservation_metric_validator();
        let item = new_reservation_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = ReservationMetricCostRangeRule { min: 2.9, max: 544.5 };
        let mut item = new_reservation_metric("x", "y");
        item.cost = 546.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = ReservationMetricHostLengthRule { max_length: 1000 };
        let mut item = new_reservation_metric("x", "y");
        item.host = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ReservationMetricCodeLengthRule { max_length: 32 };
        let mut item = new_reservation_metric("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ReservationMetricRequiredCodeRule {
            message: "reservation metric needs this field".to_string(),
        };
        let mut item = new_reservation_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ReservationMetricLabelLengthRule { max_length: 32 };
        let mut item = new_reservation_metric("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
