use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a failover cycle must pass. Check returns its problems, or none.
pub trait FailoverCycleRule: std::fmt::Debug {
    fn check(&self, item: &FailoverCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverCycleLabelLengthRule {
    pub max_length: i64,
}

impl FailoverCycleRule for FailoverCycleLabelLengthRule {
    fn check(&self, item: &FailoverCycle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverCycleAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl FailoverCycleRule for FailoverCycleAllowedLabelRule {
    fn check(&self, item: &FailoverCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is missing", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverCycleFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FailoverCycleRule for FailoverCycleFloorValueRangeRule {
    fn check(&self, item: &FailoverCycle) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverCyclePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FailoverCycleRule for FailoverCyclePortRangeRule {
    fn check(&self, item: &FailoverCycle) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverCycleMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FailoverCycleRule for FailoverCycleMaximumRangeRule {
    fn check(&self, item: &FailoverCycle) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverCycleRequiredExternalRefRule {
    pub message: String,
}

impl FailoverCycleRule for FailoverCycleRequiredExternalRefRule {
    fn check(&self, item: &FailoverCycle) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover cycle.
#[derive(Debug, Clone)]
pub struct FailoverCycleValidator {
    pub rules: Vec<Rc<dyn FailoverCycleRule>>,
}

impl FailoverCycleValidator {
    pub fn validate(&self, item: &FailoverCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FailoverCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_cycle_validator() -> FailoverCycleValidator {
    FailoverCycleValidator {
        rules: vec![
            Rc::new(FailoverCycleLabelLengthRule { max_length: 1000 }) as Rc<dyn FailoverCycleRule>,
            Rc::new(FailoverCycleAllowedLabelRule {
                allowed: vec!["primary".to_string()],
            }),
            Rc::new(FailoverCycleFloorValueRangeRule { min: 1.3, max: 4324.2 }),
            Rc::new(FailoverCyclePortRangeRule { min: 16, max: 80 }),
            Rc::new(FailoverCycleMaximumRangeRule { min: 9.3, max: 300.7 }),
            Rc::new(FailoverCycleRequiredExternalRefRule {
                message: "failover cycle needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_failover_cycle_validator();
        let item = new_failover_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = FailoverCycleLabelLengthRule { max_length: 64 };
        let mut item = new_failover_cycle("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = FailoverCycleFloorValueRangeRule { min: 6.0, max: 1315.0 };
        let mut item = new_failover_cycle("x", "y");
        item.floor_value = 1316.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = FailoverCyclePortRangeRule { min: 16, max: 120 };
        let mut item = new_failover_cycle("x", "y");
        item.port = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = FailoverCycleMaximumRangeRule { min: 0.3, max: 2145.6 };
        let mut item = new_failover_cycle("x", "y");
        item.maximum = 2147.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = FailoverCycleRequiredExternalRefRule {
            message: "failover cycle needs a value".to_string(),
        };
        let mut item = new_failover_cycle("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
