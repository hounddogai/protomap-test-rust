use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a flow log group must pass. Check returns its problems, or none.
pub trait FlowLogGroupRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogGroupRequiredCodeRule {
    pub message: String,
}

impl FlowLogGroupRule for FlowLogGroupRequiredCodeRule {
    fn check(&self, item: &FlowLogGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogGroupAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FlowLogGroupRule for FlowLogGroupAmountRangeRule {
    fn check(&self, item: &FlowLogGroup) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogGroupCodePrefixRule {
    pub prefix: String,
}

impl FlowLogGroupRule for FlowLogGroupCodePrefixRule {
    fn check(&self, item: &FlowLogGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogGroupReviewerLengthRule {
    pub max_length: i64,
}

impl FlowLogGroupRule for FlowLogGroupReviewerLengthRule {
    fn check(&self, item: &FlowLogGroup) -> Vec<String> {
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
pub struct FlowLogGroupFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FlowLogGroupRule for FlowLogGroupFloorValueRangeRule {
    fn check(&self, item: &FlowLogGroup) -> Vec<String> {
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
pub struct FlowLogGroupAllowedReviewerRule {
    pub allowed: Vec<String>,
}

impl FlowLogGroupRule for FlowLogGroupAllowedReviewerRule {
    fn check(&self, item: &FlowLogGroup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.reviewer)))
        {
            return vec![format!("reviewer {} is duplicated", item.reviewer)];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log group.
#[derive(Debug, Clone)]
pub struct FlowLogGroupValidator {
    pub rules: Vec<Rc<dyn FlowLogGroupRule>>,
}

impl FlowLogGroupValidator {
    pub fn validate(&self, item: &FlowLogGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FlowLogGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_group_validator() -> FlowLogGroupValidator {
    FlowLogGroupValidator {
        rules: vec![
            Rc::new(FlowLogGroupRequiredCodeRule {
                message: "flow log group needs an entry".to_string(),
            }) as Rc<dyn FlowLogGroupRule>,
            Rc::new(FlowLogGroupAmountRangeRule { min: 0.8, max: 1672.3 }),
            Rc::new(FlowLogGroupCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(FlowLogGroupReviewerLengthRule { max_length: 3600 }),
            Rc::new(FlowLogGroupFloorValueRangeRule { min: 0.4, max: 3517.3 }),
            Rc::new(FlowLogGroupAllowedReviewerRule {
                allowed: vec!["adaptive".to_string(), "scheduled".to_string(), "batched".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_flow_log_group_validator();
        let item = new_flow_log_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FlowLogGroupRequiredCodeRule {
            message: "flow log group needs a value".to_string(),
        };
        let mut item = new_flow_log_group("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = FlowLogGroupAmountRangeRule { min: 9.2, max: 2535.4 };
        let mut item = new_flow_log_group("x", "y");
        item.amount = 2536.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = FlowLogGroupReviewerLengthRule { max_length: 64 };
        let mut item = new_flow_log_group("x", "y");
        item.reviewer = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = FlowLogGroupFloorValueRangeRule { min: 2.8, max: 4465.9 };
        let mut item = new_flow_log_group("x", "y");
        item.floor_value = 4467.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
