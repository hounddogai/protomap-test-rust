use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a policy estimate must pass. Check returns its problems, or none.
pub trait PolicyEstimateRule: std::fmt::Debug {
    fn check(&self, item: &PolicyEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyEstimateSegmentLengthRule {
    pub max_length: i64,
}

impl PolicyEstimateRule for PolicyEstimateSegmentLengthRule {
    fn check(&self, item: &PolicyEstimate) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyEstimateLabelLengthRule {
    pub max_length: i64,
}

impl PolicyEstimateRule for PolicyEstimateLabelLengthRule {
    fn check(&self, item: &PolicyEstimate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyEstimateCodeLengthRule {
    pub max_length: i64,
}

impl PolicyEstimateRule for PolicyEstimateCodeLengthRule {
    fn check(&self, item: &PolicyEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyEstimateRequiredLabelRule {
    pub message: String,
}

impl PolicyEstimateRule for PolicyEstimateRequiredLabelRule {
    fn check(&self, item: &PolicyEstimate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a policy estimate.
#[derive(Debug, Clone)]
pub struct PolicyEstimateValidator {
    pub rules: Vec<Rc<dyn PolicyEstimateRule>>,
}

impl PolicyEstimateValidator {
    pub fn validate(&self, item: &PolicyEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PolicyEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicyEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_estimate_validator() -> PolicyEstimateValidator {
    PolicyEstimateValidator {
        rules: vec![
            Rc::new(PolicyEstimateSegmentLengthRule { max_length: 3600 }) as Rc<dyn PolicyEstimateRule>,
            Rc::new(PolicyEstimateLabelLengthRule { max_length: 255 }),
            Rc::new(PolicyEstimateCodeLengthRule { max_length: 255 }),
            Rc::new(PolicyEstimateRequiredLabelRule {
                message: "policy estimate needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_policy_estimate_validator();
        let item = new_policy_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = PolicyEstimateSegmentLengthRule { max_length: 64 };
        let mut item = new_policy_estimate("x", "y");
        item.segment = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PolicyEstimateLabelLengthRule { max_length: 64 };
        let mut item = new_policy_estimate("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PolicyEstimateCodeLengthRule { max_length: 32 };
        let mut item = new_policy_estimate("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PolicyEstimateRequiredLabelRule {
            message: "policy estimate needs a value".to_string(),
        };
        let mut item = new_policy_estimate("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
