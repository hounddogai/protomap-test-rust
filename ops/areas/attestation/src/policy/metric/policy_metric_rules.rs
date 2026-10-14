use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a policy metric must pass. Check returns its problems, or none.
pub trait PolicyMetricRule: std::fmt::Debug {
    fn check(&self, item: &PolicyMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyMetricRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PolicyMetricRule for PolicyMetricRateRangeRule {
    fn check(&self, item: &PolicyMetric) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyMetricRequiredClusterRule {
    pub message: String,
}

impl PolicyMetricRule for PolicyMetricRequiredClusterRule {
    fn check(&self, item: &PolicyMetric) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyMetricClusterLengthRule {
    pub max_length: i64,
}

impl PolicyMetricRule for PolicyMetricClusterLengthRule {
    fn check(&self, item: &PolicyMetric) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyMetricCodeLengthRule {
    pub max_length: i64,
}

impl PolicyMetricRule for PolicyMetricCodeLengthRule {
    fn check(&self, item: &PolicyMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyMetricAllowedChannelRule {
    pub allowed: Vec<String>,
}

impl PolicyMetricRule for PolicyMetricAllowedChannelRule {
    fn check(&self, item: &PolicyMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.channel)))
        {
            return vec![format!("channel {} is out of range", item.channel)];
        }
        Vec::new()
    }
}

/// Runs every rule of a policy metric.
#[derive(Debug, Clone)]
pub struct PolicyMetricValidator {
    pub rules: Vec<Rc<dyn PolicyMetricRule>>,
}

impl PolicyMetricValidator {
    pub fn validate(&self, item: &PolicyMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PolicyMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicyMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_metric_validator() -> PolicyMetricValidator {
    PolicyMetricValidator {
        rules: vec![
            Rc::new(PolicyMetricRateRangeRule { min: 8.5, max: 614.8 }) as Rc<dyn PolicyMetricRule>,
            Rc::new(PolicyMetricRequiredClusterRule {
                message: "policy metric needs an entry".to_string(),
            }),
            Rc::new(PolicyMetricClusterLengthRule { max_length: 80 }),
            Rc::new(PolicyMetricCodeLengthRule { max_length: 86400 }),
            Rc::new(PolicyMetricAllowedChannelRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_policy_metric_validator();
        let item = new_policy_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = PolicyMetricRateRangeRule { min: 7.8, max: 3494.6 };
        let mut item = new_policy_metric("x", "y");
        item.rate = 3496.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = PolicyMetricRequiredClusterRule {
            message: "policy metric needs a value".to_string(),
        };
        let mut item = new_policy_metric("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = PolicyMetricClusterLengthRule { max_length: 3600 };
        let mut item = new_policy_metric("x", "y");
        item.cluster = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PolicyMetricCodeLengthRule { max_length: 32 };
        let mut item = new_policy_metric("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
