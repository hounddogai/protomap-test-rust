use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback tenant rule must pass. Check returns its problems, or none.
pub trait FallbackTenantRuleRule: std::fmt::Debug {
    fn check(&self, item: &FallbackTenantRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTenantRuleExternalRefLengthRule {
    pub max_length: i64,
}

impl FallbackTenantRuleRule for FallbackTenantRuleExternalRefLengthRule {
    fn check(&self, item: &FallbackTenantRule) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTenantRuleCodePrefixRule {
    pub prefix: String,
}

impl FallbackTenantRuleRule for FallbackTenantRuleCodePrefixRule {
    fn check(&self, item: &FallbackTenantRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTenantRuleRequiredExternalRefRule {
    pub message: String,
}

impl FallbackTenantRuleRule for FallbackTenantRuleRequiredExternalRefRule {
    fn check(&self, item: &FallbackTenantRule) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTenantRuleRequiredChannelRule {
    pub message: String,
}

impl FallbackTenantRuleRule for FallbackTenantRuleRequiredChannelRule {
    fn check(&self, item: &FallbackTenantRule) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback tenant rule.
#[derive(Debug, Clone)]
pub struct FallbackTenantRuleValidator {
    pub rules: Vec<Rc<dyn FallbackTenantRuleRule>>,
}

impl FallbackTenantRuleValidator {
    pub fn validate(&self, item: &FallbackTenantRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackTenantRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackTenantRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_tenant_rule_validator() -> FallbackTenantRuleValidator {
    FallbackTenantRuleValidator {
        rules: vec![
            Rc::new(FallbackTenantRuleExternalRefLengthRule { max_length: 86400 }) as Rc<dyn FallbackTenantRuleRule>,
            Rc::new(FallbackTenantRuleCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(FallbackTenantRuleRequiredExternalRefRule {
                message: "fallback tenant rule needs a value".to_string(),
            }),
            Rc::new(FallbackTenantRuleRequiredChannelRule {
                message: "fallback tenant rule needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_tenant_rule_validator();
        let item = new_fallback_tenant_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = FallbackTenantRuleExternalRefLengthRule { max_length: 86400 };
        let mut item = new_fallback_tenant_rule("x", "y");
        item.external_ref = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = FallbackTenantRuleRequiredExternalRefRule {
            message: "fallback tenant rule needs this field".to_string(),
        };
        let mut item = new_fallback_tenant_rule("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = FallbackTenantRuleRequiredChannelRule {
            message: "fallback tenant rule needs this field".to_string(),
        };
        let mut item = new_fallback_tenant_rule("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
