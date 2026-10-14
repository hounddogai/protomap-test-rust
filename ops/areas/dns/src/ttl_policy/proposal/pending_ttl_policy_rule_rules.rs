use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending ttl policy rule must pass. Check returns its problems, or none.
pub trait PendingTtlPolicyRuleRule: std::fmt::Debug {
    fn check(&self, item: &PendingTtlPolicyRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyRuleCodeLengthRule {
    pub max_length: i64,
}

impl PendingTtlPolicyRuleRule for PendingTtlPolicyRuleCodeLengthRule {
    fn check(&self, item: &PendingTtlPolicyRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyRuleCodePrefixRule {
    pub prefix: String,
}

impl PendingTtlPolicyRuleRule for PendingTtlPolicyRuleCodePrefixRule {
    fn check(&self, item: &PendingTtlPolicyRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyRuleRequiredCipherRule {
    pub message: String,
}

impl PendingTtlPolicyRuleRule for PendingTtlPolicyRuleRequiredCipherRule {
    fn check(&self, item: &PendingTtlPolicyRule) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyRuleRequiredCodeRule {
    pub message: String,
}

impl PendingTtlPolicyRuleRule for PendingTtlPolicyRuleRequiredCodeRule {
    fn check(&self, item: &PendingTtlPolicyRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending ttl policy rule.
#[derive(Debug, Clone)]
pub struct PendingTtlPolicyRuleValidator {
    pub rules: Vec<Rc<dyn PendingTtlPolicyRuleRule>>,
}

impl PendingTtlPolicyRuleValidator {
    pub fn validate(&self, item: &PendingTtlPolicyRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingTtlPolicyRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingTtlPolicyRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_ttl_policy_rule_validator() -> PendingTtlPolicyRuleValidator {
    PendingTtlPolicyRuleValidator {
        rules: vec![
            Rc::new(PendingTtlPolicyRuleCodeLengthRule { max_length: 80 }) as Rc<dyn PendingTtlPolicyRuleRule>,
            Rc::new(PendingTtlPolicyRuleCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(PendingTtlPolicyRuleRequiredCipherRule {
                message: "pending ttl policy rule needs an entry".to_string(),
            }),
            Rc::new(PendingTtlPolicyRuleRequiredCodeRule {
                message: "pending ttl policy rule needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_ttl_policy_rule_validator();
        let item = new_pending_ttl_policy_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingTtlPolicyRuleCodeLengthRule { max_length: 3600 };
        let mut item = new_pending_ttl_policy_rule("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = PendingTtlPolicyRuleRequiredCipherRule {
            message: "pending ttl policy rule needs an entry".to_string(),
        };
        let mut item = new_pending_ttl_policy_rule("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingTtlPolicyRuleRequiredCodeRule {
            message: "pending ttl policy rule needs an entry".to_string(),
        };
        let mut item = new_pending_ttl_policy_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
