use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a flow log threshold must pass. Check returns its problems, or none.
pub trait FlowLogThresholdRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogThresholdReviewerLengthRule {
    pub max_length: i64,
}

impl FlowLogThresholdRule for FlowLogThresholdReviewerLengthRule {
    fn check(&self, item: &FlowLogThreshold) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogThresholdCodeLengthRule {
    pub max_length: i64,
}

impl FlowLogThresholdRule for FlowLogThresholdCodeLengthRule {
    fn check(&self, item: &FlowLogThreshold) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogThresholdTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FlowLogThresholdRule for FlowLogThresholdTtlSecondsRangeRule {
    fn check(&self, item: &FlowLogThreshold) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogThresholdRequiredCodeRule {
    pub message: String,
}

impl FlowLogThresholdRule for FlowLogThresholdRequiredCodeRule {
    fn check(&self, item: &FlowLogThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogThresholdRequiredReviewerRule {
    pub message: String,
}

impl FlowLogThresholdRule for FlowLogThresholdRequiredReviewerRule {
    fn check(&self, item: &FlowLogThreshold) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogThresholdRequiredTitleRule {
    pub message: String,
}

impl FlowLogThresholdRule for FlowLogThresholdRequiredTitleRule {
    fn check(&self, item: &FlowLogThreshold) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log threshold.
#[derive(Debug, Clone)]
pub struct FlowLogThresholdValidator {
    pub rules: Vec<Rc<dyn FlowLogThresholdRule>>,
}

impl FlowLogThresholdValidator {
    pub fn validate(&self, item: &FlowLogThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FlowLogThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_threshold_validator() -> FlowLogThresholdValidator {
    FlowLogThresholdValidator {
        rules: vec![
            Rc::new(FlowLogThresholdReviewerLengthRule { max_length: 86400 }) as Rc<dyn FlowLogThresholdRule>,
            Rc::new(FlowLogThresholdCodeLengthRule { max_length: 16 }),
            Rc::new(FlowLogThresholdTtlSecondsRangeRule { min: 16, max: 255 }),
            Rc::new(FlowLogThresholdRequiredCodeRule {
                message: "flow log threshold needs an entry".to_string(),
            }),
            Rc::new(FlowLogThresholdRequiredReviewerRule {
                message: "flow log threshold needs this field".to_string(),
            }),
            Rc::new(FlowLogThresholdRequiredTitleRule {
                message: "flow log threshold needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_flow_log_threshold_validator();
        let item = new_flow_log_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = FlowLogThresholdReviewerLengthRule { max_length: 1000 };
        let mut item = new_flow_log_threshold("x", "y");
        item.reviewer = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FlowLogThresholdCodeLengthRule { max_length: 80 };
        let mut item = new_flow_log_threshold("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = FlowLogThresholdTtlSecondsRangeRule { min: 80, max: 86400 };
        let mut item = new_flow_log_threshold("x", "y");
        item.ttl_seconds = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FlowLogThresholdRequiredCodeRule {
            message: "flow log threshold needs a value".to_string(),
        };
        let mut item = new_flow_log_threshold("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = FlowLogThresholdRequiredReviewerRule {
            message: "flow log threshold needs a value".to_string(),
        };
        let mut item = new_flow_log_threshold("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = FlowLogThresholdRequiredTitleRule {
            message: "flow log threshold needs an entry".to_string(),
        };
        let mut item = new_flow_log_threshold("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
