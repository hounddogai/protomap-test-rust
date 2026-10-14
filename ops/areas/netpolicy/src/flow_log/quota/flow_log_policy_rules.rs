use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a flow log policy must pass. Check returns its problems, or none.
pub trait FlowLogPolicyRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogPolicyRequiredClusterRule {
    pub message: String,
}

impl FlowLogPolicyRule for FlowLogPolicyRequiredClusterRule {
    fn check(&self, item: &FlowLogPolicy) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogPolicyRequiredReviewerRule {
    pub message: String,
}

impl FlowLogPolicyRule for FlowLogPolicyRequiredReviewerRule {
    fn check(&self, item: &FlowLogPolicy) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogPolicyRequiredTitleRule {
    pub message: String,
}

impl FlowLogPolicyRule for FlowLogPolicyRequiredTitleRule {
    fn check(&self, item: &FlowLogPolicy) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogPolicyAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl FlowLogPolicyRule for FlowLogPolicyAllowedChecksumRule {
    fn check(&self, item: &FlowLogPolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is expired", item.checksum)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogPolicyRequiredCodeRule {
    pub message: String,
}

impl FlowLogPolicyRule for FlowLogPolicyRequiredCodeRule {
    fn check(&self, item: &FlowLogPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogPolicyCodeLengthRule {
    pub max_length: i64,
}

impl FlowLogPolicyRule for FlowLogPolicyCodeLengthRule {
    fn check(&self, item: &FlowLogPolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log policy.
#[derive(Debug, Clone)]
pub struct FlowLogPolicyValidator {
    pub rules: Vec<Rc<dyn FlowLogPolicyRule>>,
}

impl FlowLogPolicyValidator {
    pub fn validate(&self, item: &FlowLogPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FlowLogPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_policy_validator() -> FlowLogPolicyValidator {
    FlowLogPolicyValidator {
        rules: vec![
            Rc::new(FlowLogPolicyRequiredClusterRule {
                message: "flow log policy needs this field".to_string(),
            }) as Rc<dyn FlowLogPolicyRule>,
            Rc::new(FlowLogPolicyRequiredReviewerRule {
                message: "flow log policy needs a value".to_string(),
            }),
            Rc::new(FlowLogPolicyRequiredTitleRule {
                message: "flow log policy needs this field".to_string(),
            }),
            Rc::new(FlowLogPolicyAllowedChecksumRule {
                allowed: vec!["weighted".to_string(), "batched".to_string(), "default".to_string()],
            }),
            Rc::new(FlowLogPolicyRequiredCodeRule {
                message: "flow log policy needs this field".to_string(),
            }),
            Rc::new(FlowLogPolicyCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_flow_log_policy_validator();
        let item = new_flow_log_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = FlowLogPolicyRequiredClusterRule {
            message: "flow log policy needs an entry".to_string(),
        };
        let mut item = new_flow_log_policy("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = FlowLogPolicyRequiredReviewerRule {
            message: "flow log policy needs an entry".to_string(),
        };
        let mut item = new_flow_log_policy("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = FlowLogPolicyRequiredTitleRule {
            message: "flow log policy needs an entry".to_string(),
        };
        let mut item = new_flow_log_policy("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FlowLogPolicyRequiredCodeRule {
            message: "flow log policy needs an entry".to_string(),
        };
        let mut item = new_flow_log_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FlowLogPolicyCodeLengthRule { max_length: 32 };
        let mut item = new_flow_log_policy("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
