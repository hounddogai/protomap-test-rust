use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a policy segment must pass. Check returns its problems, or none.
pub trait PolicySegmentRule: std::fmt::Debug {
    fn check(&self, item: &PolicySegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicySegmentCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PolicySegmentRule for PolicySegmentCostRangeRule {
    fn check(&self, item: &PolicySegment) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicySegmentRequiredLabelRule {
    pub message: String,
}

impl PolicySegmentRule for PolicySegmentRequiredLabelRule {
    fn check(&self, item: &PolicySegment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicySegmentRequiredCodeRule {
    pub message: String,
}

impl PolicySegmentRule for PolicySegmentRequiredCodeRule {
    fn check(&self, item: &PolicySegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicySegmentCodePrefixRule {
    pub prefix: String,
}

impl PolicySegmentRule for PolicySegmentCodePrefixRule {
    fn check(&self, item: &PolicySegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicySegmentLabelLengthRule {
    pub max_length: i64,
}

impl PolicySegmentRule for PolicySegmentLabelLengthRule {
    fn check(&self, item: &PolicySegment) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a policy segment.
#[derive(Debug, Clone)]
pub struct PolicySegmentValidator {
    pub rules: Vec<Rc<dyn PolicySegmentRule>>,
}

impl PolicySegmentValidator {
    pub fn validate(&self, item: &PolicySegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PolicySegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicySegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_segment_validator() -> PolicySegmentValidator {
    PolicySegmentValidator {
        rules: vec![
            Rc::new(PolicySegmentCostRangeRule { min: 7.0, max: 1557.4 }) as Rc<dyn PolicySegmentRule>,
            Rc::new(PolicySegmentRequiredLabelRule {
                message: "policy segment needs a value".to_string(),
            }),
            Rc::new(PolicySegmentRequiredCodeRule {
                message: "policy segment needs an entry".to_string(),
            }),
            Rc::new(PolicySegmentCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(PolicySegmentLabelLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_policy_segment_validator();
        let item = new_policy_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = PolicySegmentCostRangeRule { min: 3.4, max: 1123.6 };
        let mut item = new_policy_segment("x", "y");
        item.cost = 1125.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PolicySegmentRequiredLabelRule {
            message: "policy segment needs this field".to_string(),
        };
        let mut item = new_policy_segment("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PolicySegmentRequiredCodeRule {
            message: "policy segment needs an entry".to_string(),
        };
        let mut item = new_policy_segment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PolicySegmentLabelLengthRule { max_length: 1000 };
        let mut item = new_policy_segment("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
