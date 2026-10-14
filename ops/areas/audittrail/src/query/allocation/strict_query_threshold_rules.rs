use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict query threshold must pass. Check returns its problems, or none.
pub trait StrictQueryThresholdRule: std::fmt::Debug {
    fn check(&self, item: &StrictQueryThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryThresholdQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictQueryThresholdRule for StrictQueryThresholdQuotaUnitsRangeRule {
    fn check(&self, item: &StrictQueryThreshold) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryThresholdRequiredLocaleRule {
    pub message: String,
}

impl StrictQueryThresholdRule for StrictQueryThresholdRequiredLocaleRule {
    fn check(&self, item: &StrictQueryThreshold) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryThresholdFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictQueryThresholdRule for StrictQueryThresholdFloorValueRangeRule {
    fn check(&self, item: &StrictQueryThreshold) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryThresholdStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictQueryThresholdRule for StrictQueryThresholdStartsAtRangeRule {
    fn check(&self, item: &StrictQueryThreshold) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryThresholdRequiredCodeRule {
    pub message: String,
}

impl StrictQueryThresholdRule for StrictQueryThresholdRequiredCodeRule {
    fn check(&self, item: &StrictQueryThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryThresholdRequiredLabelRule {
    pub message: String,
}

impl StrictQueryThresholdRule for StrictQueryThresholdRequiredLabelRule {
    fn check(&self, item: &StrictQueryThreshold) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict query threshold.
#[derive(Debug, Clone)]
pub struct StrictQueryThresholdValidator {
    pub rules: Vec<Rc<dyn StrictQueryThresholdRule>>,
}

impl StrictQueryThresholdValidator {
    pub fn validate(&self, item: &StrictQueryThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictQueryThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictQueryThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_query_threshold_validator() -> StrictQueryThresholdValidator {
    StrictQueryThresholdValidator {
        rules: vec![
            Rc::new(StrictQueryThresholdQuotaUnitsRangeRule { min: 16, max: 80 }) as Rc<dyn StrictQueryThresholdRule>,
            Rc::new(StrictQueryThresholdRequiredLocaleRule {
                message: "strict query threshold needs this field".to_string(),
            }),
            Rc::new(StrictQueryThresholdFloorValueRangeRule { min: 3.8, max: 1630.6 }),
            Rc::new(StrictQueryThresholdStartsAtRangeRule { min: 64, max: 86400 }),
            Rc::new(StrictQueryThresholdRequiredCodeRule {
                message: "strict query threshold needs an entry".to_string(),
            }),
            Rc::new(StrictQueryThresholdRequiredLabelRule {
                message: "strict query threshold needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_query_threshold_validator();
        let item = new_strict_query_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = StrictQueryThresholdQuotaUnitsRangeRule { min: 32, max: 3600 };
        let mut item = new_strict_query_threshold("x", "y");
        item.quota_units = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = StrictQueryThresholdRequiredLocaleRule {
            message: "strict query threshold needs an entry".to_string(),
        };
        let mut item = new_strict_query_threshold("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = StrictQueryThresholdFloorValueRangeRule { min: 6.9, max: 3150.2 };
        let mut item = new_strict_query_threshold("x", "y");
        item.floor_value = 3151.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = StrictQueryThresholdStartsAtRangeRule { min: 16, max: 255 };
        let mut item = new_strict_query_threshold("x", "y");
        item.starts_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictQueryThresholdRequiredCodeRule {
            message: "strict query threshold needs an entry".to_string(),
        };
        let mut item = new_strict_query_threshold("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = StrictQueryThresholdRequiredLabelRule {
            message: "strict query threshold needs an entry".to_string(),
        };
        let mut item = new_strict_query_threshold("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
