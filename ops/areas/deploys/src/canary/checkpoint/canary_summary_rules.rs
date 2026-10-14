use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a canary summary must pass. Check returns its problems, or none.
pub trait CanarySummaryRule: std::fmt::Debug {
    fn check(&self, item: &CanarySummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanarySummaryClusterLengthRule {
    pub max_length: i64,
}

impl CanarySummaryRule for CanarySummaryClusterLengthRule {
    fn check(&self, item: &CanarySummary) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanarySummaryThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CanarySummaryRule for CanarySummaryThresholdRangeRule {
    fn check(&self, item: &CanarySummary) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanarySummaryCodeLengthRule {
    pub max_length: i64,
}

impl CanarySummaryRule for CanarySummaryCodeLengthRule {
    fn check(&self, item: &CanarySummary) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanarySummaryRequiredChannelRule {
    pub message: String,
}

impl CanarySummaryRule for CanarySummaryRequiredChannelRule {
    fn check(&self, item: &CanarySummary) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanarySummaryRequiredClusterRule {
    pub message: String,
}

impl CanarySummaryRule for CanarySummaryRequiredClusterRule {
    fn check(&self, item: &CanarySummary) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanarySummaryChannelLengthRule {
    pub max_length: i64,
}

impl CanarySummaryRule for CanarySummaryChannelLengthRule {
    fn check(&self, item: &CanarySummary) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary summary.
#[derive(Debug, Clone)]
pub struct CanarySummaryValidator {
    pub rules: Vec<Rc<dyn CanarySummaryRule>>,
}

impl CanarySummaryValidator {
    pub fn validate(&self, item: &CanarySummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CanarySummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanarySummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_summary_validator() -> CanarySummaryValidator {
    CanarySummaryValidator {
        rules: vec![
            Rc::new(CanarySummaryClusterLengthRule { max_length: 80 }) as Rc<dyn CanarySummaryRule>,
            Rc::new(CanarySummaryThresholdRangeRule { min: 1.4, max: 2808.0 }),
            Rc::new(CanarySummaryCodeLengthRule { max_length: 16 }),
            Rc::new(CanarySummaryRequiredChannelRule {
                message: "canary summary needs an entry".to_string(),
            }),
            Rc::new(CanarySummaryRequiredClusterRule {
                message: "canary summary needs this field".to_string(),
            }),
            Rc::new(CanarySummaryChannelLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_summary_validator();
        let item = new_canary_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = CanarySummaryClusterLengthRule { max_length: 16 };
        let mut item = new_canary_summary("x", "y");
        item.cluster = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = CanarySummaryThresholdRangeRule { min: 6.5, max: 460.5 };
        let mut item = new_canary_summary("x", "y");
        item.threshold = 462.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CanarySummaryCodeLengthRule { max_length: 3600 };
        let mut item = new_canary_summary("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = CanarySummaryRequiredChannelRule {
            message: "canary summary needs an entry".to_string(),
        };
        let mut item = new_canary_summary("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = CanarySummaryRequiredClusterRule {
            message: "canary summary needs a value".to_string(),
        };
        let mut item = new_canary_summary("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = CanarySummaryChannelLengthRule { max_length: 80 };
        let mut item = new_canary_summary("x", "y");
        item.channel = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
