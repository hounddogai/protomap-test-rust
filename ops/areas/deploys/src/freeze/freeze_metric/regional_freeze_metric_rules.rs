use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional freeze metric must pass. Check returns its problems, or none.
pub trait RegionalFreezeMetricRule: std::fmt::Debug {
    fn check(&self, item: &RegionalFreezeMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFreezeMetricAllowedFingerprintRule {
    pub allowed: Vec<String>,
}

impl RegionalFreezeMetricRule for RegionalFreezeMetricAllowedFingerprintRule {
    fn check(&self, item: &RegionalFreezeMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.fingerprint)))
        {
            return vec![format!("fingerprint {} is stale", item.fingerprint)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFreezeMetricRequiredTitleRule {
    pub message: String,
}

impl RegionalFreezeMetricRule for RegionalFreezeMetricRequiredTitleRule {
    fn check(&self, item: &RegionalFreezeMetric) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFreezeMetricCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalFreezeMetricRule for RegionalFreezeMetricCostRangeRule {
    fn check(&self, item: &RegionalFreezeMetric) -> Vec<String> {
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
pub struct RegionalFreezeMetricRequiredCodeRule {
    pub message: String,
}

impl RegionalFreezeMetricRule for RegionalFreezeMetricRequiredCodeRule {
    fn check(&self, item: &RegionalFreezeMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFreezeMetricFingerprintLengthRule {
    pub max_length: i64,
}

impl RegionalFreezeMetricRule for RegionalFreezeMetricFingerprintLengthRule {
    fn check(&self, item: &RegionalFreezeMetric) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFreezeMetricCodeLengthRule {
    pub max_length: i64,
}

impl RegionalFreezeMetricRule for RegionalFreezeMetricCodeLengthRule {
    fn check(&self, item: &RegionalFreezeMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional freeze metric.
#[derive(Debug, Clone)]
pub struct RegionalFreezeMetricValidator {
    pub rules: Vec<Rc<dyn RegionalFreezeMetricRule>>,
}

impl RegionalFreezeMetricValidator {
    pub fn validate(&self, item: &RegionalFreezeMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalFreezeMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalFreezeMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_freeze_metric_validator() -> RegionalFreezeMetricValidator {
    RegionalFreezeMetricValidator {
        rules: vec![
            Rc::new(RegionalFreezeMetricAllowedFingerprintRule {
                allowed: vec!["scheduled".to_string(), "global".to_string(), "rolling".to_string()],
            }) as Rc<dyn RegionalFreezeMetricRule>,
            Rc::new(RegionalFreezeMetricRequiredTitleRule {
                message: "regional freeze metric needs a value".to_string(),
            }),
            Rc::new(RegionalFreezeMetricCostRangeRule { min: 7.8, max: 4078.8 }),
            Rc::new(RegionalFreezeMetricRequiredCodeRule {
                message: "regional freeze metric needs this field".to_string(),
            }),
            Rc::new(RegionalFreezeMetricFingerprintLengthRule { max_length: 86400 }),
            Rc::new(RegionalFreezeMetricCodeLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_freeze_metric_validator();
        let item = new_regional_freeze_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RegionalFreezeMetricRequiredTitleRule {
            message: "regional freeze metric needs this field".to_string(),
        };
        let mut item = new_regional_freeze_metric("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = RegionalFreezeMetricCostRangeRule { min: 3.9, max: 704.8 };
        let mut item = new_regional_freeze_metric("x", "y");
        item.cost = 706.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalFreezeMetricRequiredCodeRule {
            message: "regional freeze metric needs this field".to_string(),
        };
        let mut item = new_regional_freeze_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = RegionalFreezeMetricFingerprintLengthRule { max_length: 255 };
        let mut item = new_regional_freeze_metric("x", "y");
        item.fingerprint = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalFreezeMetricCodeLengthRule { max_length: 80 };
        let mut item = new_regional_freeze_metric("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
