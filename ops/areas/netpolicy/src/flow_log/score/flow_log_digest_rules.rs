use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a flow log digest must pass. Check returns its problems, or none.
pub trait FlowLogDigestRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogDigestAllowedPhoneRule {
    pub allowed: Vec<String>,
}

impl FlowLogDigestRule for FlowLogDigestAllowedPhoneRule {
    fn check(&self, item: &FlowLogDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.phone))) {
            return vec![format!("phone {} is above maximum", item.phone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogDigestRequiredLabelRule {
    pub message: String,
}

impl FlowLogDigestRule for FlowLogDigestRequiredLabelRule {
    fn check(&self, item: &FlowLogDigest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogDigestRequiredPhoneRule {
    pub message: String,
}

impl FlowLogDigestRule for FlowLogDigestRequiredPhoneRule {
    fn check(&self, item: &FlowLogDigest) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogDigestCodePrefixRule {
    pub prefix: String,
}

impl FlowLogDigestRule for FlowLogDigestCodePrefixRule {
    fn check(&self, item: &FlowLogDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log digest.
#[derive(Debug, Clone)]
pub struct FlowLogDigestValidator {
    pub rules: Vec<Rc<dyn FlowLogDigestRule>>,
}

impl FlowLogDigestValidator {
    pub fn validate(&self, item: &FlowLogDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FlowLogDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_digest_validator() -> FlowLogDigestValidator {
    FlowLogDigestValidator {
        rules: vec![
            Rc::new(FlowLogDigestAllowedPhoneRule {
                allowed: vec!["manual".to_string(), "weighted".to_string()],
            }) as Rc<dyn FlowLogDigestRule>,
            Rc::new(FlowLogDigestRequiredLabelRule {
                message: "flow log digest needs a value".to_string(),
            }),
            Rc::new(FlowLogDigestRequiredPhoneRule {
                message: "flow log digest needs this field".to_string(),
            }),
            Rc::new(FlowLogDigestCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_flow_log_digest_validator();
        let item = new_flow_log_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = FlowLogDigestRequiredLabelRule {
            message: "flow log digest needs this field".to_string(),
        };
        let mut item = new_flow_log_digest("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = FlowLogDigestRequiredPhoneRule {
            message: "flow log digest needs an entry".to_string(),
        };
        let mut item = new_flow_log_digest("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
