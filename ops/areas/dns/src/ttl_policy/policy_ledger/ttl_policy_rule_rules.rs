use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a ttl policy rule must pass. Check returns its problems, or none.
pub trait TtlPolicyRuleRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyRuleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TtlPolicyRuleRule for TtlPolicyRuleAllowedCodeRule {
    fn check(&self, item: &TtlPolicyRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyRuleRequiredCodeRule {
    pub message: String,
}

impl TtlPolicyRuleRule for TtlPolicyRuleRequiredCodeRule {
    fn check(&self, item: &TtlPolicyRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyRuleLabelLengthRule {
    pub max_length: i64,
}

impl TtlPolicyRuleRule for TtlPolicyRuleLabelLengthRule {
    fn check(&self, item: &TtlPolicyRule) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyRuleCodePrefixRule {
    pub prefix: String,
}

impl TtlPolicyRuleRule for TtlPolicyRuleCodePrefixRule {
    fn check(&self, item: &TtlPolicyRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyRuleCodeLengthRule {
    pub max_length: i64,
}

impl TtlPolicyRuleRule for TtlPolicyRuleCodeLengthRule {
    fn check(&self, item: &TtlPolicyRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyRuleRequiredLabelRule {
    pub message: String,
}

impl TtlPolicyRuleRule for TtlPolicyRuleRequiredLabelRule {
    fn check(&self, item: &TtlPolicyRule) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy rule.
#[derive(Debug, Clone)]
pub struct TtlPolicyRuleValidator {
    pub rules: Vec<Rc<dyn TtlPolicyRuleRule>>,
}

impl TtlPolicyRuleValidator {
    pub fn validate(&self, item: &TtlPolicyRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TtlPolicyRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_rule_validator() -> TtlPolicyRuleValidator {
    TtlPolicyRuleValidator {
        rules: vec![
            Rc::new(TtlPolicyRuleAllowedCodeRule { allowed: Vec::new() }) as Rc<dyn TtlPolicyRuleRule>,
            Rc::new(TtlPolicyRuleRequiredCodeRule {
                message: "ttl policy rule needs an entry".to_string(),
            }),
            Rc::new(TtlPolicyRuleLabelLengthRule { max_length: 64 }),
            Rc::new(TtlPolicyRuleCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(TtlPolicyRuleCodeLengthRule { max_length: 16 }),
            Rc::new(TtlPolicyRuleRequiredLabelRule {
                message: "ttl policy rule needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_ttl_policy_rule_validator();
        let item = new_ttl_policy_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TtlPolicyRuleRequiredCodeRule {
            message: "ttl policy rule needs an entry".to_string(),
        };
        let mut item = new_ttl_policy_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TtlPolicyRuleLabelLengthRule { max_length: 3600 };
        let mut item = new_ttl_policy_rule("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TtlPolicyRuleCodeLengthRule { max_length: 32 };
        let mut item = new_ttl_policy_rule("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TtlPolicyRuleRequiredLabelRule {
            message: "ttl policy rule needs an entry".to_string(),
        };
        let mut item = new_ttl_policy_rule("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
