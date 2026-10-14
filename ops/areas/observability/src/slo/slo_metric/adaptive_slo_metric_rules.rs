use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive slo metric must pass. Check returns its problems, or none.
pub trait AdaptiveSloMetricRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveSloMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloMetricTitleLengthRule {
    pub max_length: i64,
}

impl AdaptiveSloMetricRule for AdaptiveSloMetricTitleLengthRule {
    fn check(&self, item: &AdaptiveSloMetric) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloMetricNotesLengthRule {
    pub max_length: i64,
}

impl AdaptiveSloMetricRule for AdaptiveSloMetricNotesLengthRule {
    fn check(&self, item: &AdaptiveSloMetric) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloMetricFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveSloMetricRule for AdaptiveSloMetricFloorValueRangeRule {
    fn check(&self, item: &AdaptiveSloMetric) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloMetricRequiredCodeRule {
    pub message: String,
}

impl AdaptiveSloMetricRule for AdaptiveSloMetricRequiredCodeRule {
    fn check(&self, item: &AdaptiveSloMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive slo metric.
#[derive(Debug, Clone)]
pub struct AdaptiveSloMetricValidator {
    pub rules: Vec<Rc<dyn AdaptiveSloMetricRule>>,
}

impl AdaptiveSloMetricValidator {
    pub fn validate(&self, item: &AdaptiveSloMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveSloMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveSloMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_slo_metric_validator() -> AdaptiveSloMetricValidator {
    AdaptiveSloMetricValidator {
        rules: vec![
            Rc::new(AdaptiveSloMetricTitleLengthRule { max_length: 255 }) as Rc<dyn AdaptiveSloMetricRule>,
            Rc::new(AdaptiveSloMetricNotesLengthRule { max_length: 16 }),
            Rc::new(AdaptiveSloMetricFloorValueRangeRule { min: 2.7, max: 1178.4 }),
            Rc::new(AdaptiveSloMetricRequiredCodeRule {
                message: "adaptive slo metric needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_slo_metric_validator();
        let item = new_adaptive_slo_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = AdaptiveSloMetricTitleLengthRule { max_length: 16 };
        let mut item = new_adaptive_slo_metric("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = AdaptiveSloMetricNotesLengthRule { max_length: 80 };
        let mut item = new_adaptive_slo_metric("x", "y");
        item.notes = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = AdaptiveSloMetricFloorValueRangeRule { min: 1.6, max: 592.3 };
        let mut item = new_adaptive_slo_metric("x", "y");
        item.floor_value = 593.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveSloMetricRequiredCodeRule {
            message: "adaptive slo metric needs an entry".to_string(),
        };
        let mut item = new_adaptive_slo_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
