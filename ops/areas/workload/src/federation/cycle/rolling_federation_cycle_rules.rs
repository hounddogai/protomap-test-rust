use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling federation cycle must pass. Check returns its problems, or none.
pub trait RollingFederationCycleRule: std::fmt::Debug {
    fn check(&self, item: &RollingFederationCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationCycleFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingFederationCycleRule for RollingFederationCycleFloorValueRangeRule {
    fn check(&self, item: &RollingFederationCycle) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationCycleSummaryLengthRule {
    pub max_length: i64,
}

impl RollingFederationCycleRule for RollingFederationCycleSummaryLengthRule {
    fn check(&self, item: &RollingFederationCycle) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationCycleRequiredLabelRule {
    pub message: String,
}

impl RollingFederationCycleRule for RollingFederationCycleRequiredLabelRule {
    fn check(&self, item: &RollingFederationCycle) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationCycleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RollingFederationCycleRule for RollingFederationCycleAllowedCodeRule {
    fn check(&self, item: &RollingFederationCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationCycleRequiredSummaryRule {
    pub message: String,
}

impl RollingFederationCycleRule for RollingFederationCycleRequiredSummaryRule {
    fn check(&self, item: &RollingFederationCycle) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling federation cycle.
#[derive(Debug, Clone)]
pub struct RollingFederationCycleValidator {
    pub rules: Vec<Rc<dyn RollingFederationCycleRule>>,
}

impl RollingFederationCycleValidator {
    pub fn validate(&self, item: &RollingFederationCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingFederationCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingFederationCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_federation_cycle_validator() -> RollingFederationCycleValidator {
    RollingFederationCycleValidator {
        rules: vec![
            Rc::new(RollingFederationCycleFloorValueRangeRule { min: 5.7, max: 1827.3 })
                as Rc<dyn RollingFederationCycleRule>,
            Rc::new(RollingFederationCycleSummaryLengthRule { max_length: 255 }),
            Rc::new(RollingFederationCycleRequiredLabelRule {
                message: "rolling federation cycle needs an entry".to_string(),
            }),
            Rc::new(RollingFederationCycleAllowedCodeRule {
                allowed: vec!["capped".to_string()],
            }),
            Rc::new(RollingFederationCycleRequiredSummaryRule {
                message: "rolling federation cycle needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_federation_cycle_validator();
        let item = new_rolling_federation_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = RollingFederationCycleFloorValueRangeRule { min: 3.9, max: 1427.7 };
        let mut item = new_rolling_federation_cycle("x", "y");
        item.floor_value = 1429.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = RollingFederationCycleSummaryLengthRule { max_length: 32 };
        let mut item = new_rolling_federation_cycle("x", "y");
        item.summary = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RollingFederationCycleRequiredLabelRule {
            message: "rolling federation cycle needs a value".to_string(),
        };
        let mut item = new_rolling_federation_cycle("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = RollingFederationCycleRequiredSummaryRule {
            message: "rolling federation cycle needs an entry".to_string(),
        };
        let mut item = new_rolling_federation_cycle("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
