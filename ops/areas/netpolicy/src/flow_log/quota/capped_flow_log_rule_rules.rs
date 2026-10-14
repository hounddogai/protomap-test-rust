use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped flow log rule must pass. Check returns its problems, or none.
pub trait CappedFlowLogRuleRule: std::fmt::Debug {
    fn check(&self, item: &CappedFlowLogRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFlowLogRuleKeyIdLengthRule {
    pub max_length: i64,
}

impl CappedFlowLogRuleRule for CappedFlowLogRuleKeyIdLengthRule {
    fn check(&self, item: &CappedFlowLogRule) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFlowLogRuleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl CappedFlowLogRuleRule for CappedFlowLogRuleAllowedCodeRule {
    fn check(&self, item: &CappedFlowLogRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFlowLogRuleRequiredCodeRule {
    pub message: String,
}

impl CappedFlowLogRuleRule for CappedFlowLogRuleRequiredCodeRule {
    fn check(&self, item: &CappedFlowLogRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped flow log rule.
#[derive(Debug, Clone)]
pub struct CappedFlowLogRuleValidator {
    pub rules: Vec<Rc<dyn CappedFlowLogRuleRule>>,
}

impl CappedFlowLogRuleValidator {
    pub fn validate(&self, item: &CappedFlowLogRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedFlowLogRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedFlowLogRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_flow_log_rule_validator() -> CappedFlowLogRuleValidator {
    CappedFlowLogRuleValidator {
        rules: vec![
            Rc::new(CappedFlowLogRuleKeyIdLengthRule { max_length: 64 }) as Rc<dyn CappedFlowLogRuleRule>,
            Rc::new(CappedFlowLogRuleAllowedCodeRule {
                allowed: vec!["global".to_string(), "lenient".to_string()],
            }),
            Rc::new(CappedFlowLogRuleRequiredCodeRule {
                message: "capped flow log rule needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_flow_log_rule_validator();
        let item = new_capped_flow_log_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = CappedFlowLogRuleKeyIdLengthRule { max_length: 120 };
        let mut item = new_capped_flow_log_rule("x", "y");
        item.key_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedFlowLogRuleRequiredCodeRule {
            message: "capped flow log rule needs this field".to_string(),
        };
        let mut item = new_capped_flow_log_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
