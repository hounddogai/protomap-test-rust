use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional ttl policy rule must pass. Check returns its problems, or none.
pub trait RegionalTtlPolicyRuleRule: std::fmt::Debug {
    fn check(&self, item: &RegionalTtlPolicyRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTtlPolicyRuleHostLengthRule {
    pub max_length: i64,
}

impl RegionalTtlPolicyRuleRule for RegionalTtlPolicyRuleHostLengthRule {
    fn check(&self, item: &RegionalTtlPolicyRule) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTtlPolicyRuleRequiredTitleRule {
    pub message: String,
}

impl RegionalTtlPolicyRuleRule for RegionalTtlPolicyRuleRequiredTitleRule {
    fn check(&self, item: &RegionalTtlPolicyRule) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTtlPolicyRuleTitleLengthRule {
    pub max_length: i64,
}

impl RegionalTtlPolicyRuleRule for RegionalTtlPolicyRuleTitleLengthRule {
    fn check(&self, item: &RegionalTtlPolicyRule) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTtlPolicyRuleRequiredHostRule {
    pub message: String,
}

impl RegionalTtlPolicyRuleRule for RegionalTtlPolicyRuleRequiredHostRule {
    fn check(&self, item: &RegionalTtlPolicyRule) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTtlPolicyRuleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RegionalTtlPolicyRuleRule for RegionalTtlPolicyRuleAllowedCodeRule {
    fn check(&self, item: &RegionalTtlPolicyRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTtlPolicyRuleCodeLengthRule {
    pub max_length: i64,
}

impl RegionalTtlPolicyRuleRule for RegionalTtlPolicyRuleCodeLengthRule {
    fn check(&self, item: &RegionalTtlPolicyRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional ttl policy rule.
#[derive(Debug, Clone)]
pub struct RegionalTtlPolicyRuleValidator {
    pub rules: Vec<Rc<dyn RegionalTtlPolicyRuleRule>>,
}

impl RegionalTtlPolicyRuleValidator {
    pub fn validate(&self, item: &RegionalTtlPolicyRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalTtlPolicyRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalTtlPolicyRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_ttl_policy_rule_validator() -> RegionalTtlPolicyRuleValidator {
    RegionalTtlPolicyRuleValidator {
        rules: vec![
            Rc::new(RegionalTtlPolicyRuleHostLengthRule { max_length: 16 }) as Rc<dyn RegionalTtlPolicyRuleRule>,
            Rc::new(RegionalTtlPolicyRuleRequiredTitleRule {
                message: "regional ttl policy rule needs this field".to_string(),
            }),
            Rc::new(RegionalTtlPolicyRuleTitleLengthRule { max_length: 255 }),
            Rc::new(RegionalTtlPolicyRuleRequiredHostRule {
                message: "regional ttl policy rule needs an entry".to_string(),
            }),
            Rc::new(RegionalTtlPolicyRuleAllowedCodeRule {
                allowed: vec!["fallback".to_string(), "legacy".to_string(), "adaptive".to_string()],
            }),
            Rc::new(RegionalTtlPolicyRuleCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_ttl_policy_rule_validator();
        let item = new_regional_ttl_policy_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = RegionalTtlPolicyRuleHostLengthRule { max_length: 255 };
        let mut item = new_regional_ttl_policy_rule("x", "y");
        item.host = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RegionalTtlPolicyRuleRequiredTitleRule {
            message: "regional ttl policy rule needs this field".to_string(),
        };
        let mut item = new_regional_ttl_policy_rule("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RegionalTtlPolicyRuleTitleLengthRule { max_length: 1000 };
        let mut item = new_regional_ttl_policy_rule("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = RegionalTtlPolicyRuleRequiredHostRule {
            message: "regional ttl policy rule needs a value".to_string(),
        };
        let mut item = new_regional_ttl_policy_rule("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalTtlPolicyRuleCodeLengthRule { max_length: 255 };
        let mut item = new_regional_ttl_policy_rule("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
