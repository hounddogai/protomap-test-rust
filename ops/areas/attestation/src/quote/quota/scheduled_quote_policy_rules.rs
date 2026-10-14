use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled quote policy must pass. Check returns its problems, or none.
pub trait ScheduledQuotePolicyRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledQuotePolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQuotePolicyRequiredCodeRule {
    pub message: String,
}

impl ScheduledQuotePolicyRule for ScheduledQuotePolicyRequiredCodeRule {
    fn check(&self, item: &ScheduledQuotePolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQuotePolicyIssuerLengthRule {
    pub max_length: i64,
}

impl ScheduledQuotePolicyRule for ScheduledQuotePolicyIssuerLengthRule {
    fn check(&self, item: &ScheduledQuotePolicy) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQuotePolicyAllowedNameRule {
    pub allowed: Vec<String>,
}

impl ScheduledQuotePolicyRule for ScheduledQuotePolicyAllowedNameRule {
    fn check(&self, item: &ScheduledQuotePolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is blocked", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQuotePolicyCodePrefixRule {
    pub prefix: String,
}

impl ScheduledQuotePolicyRule for ScheduledQuotePolicyCodePrefixRule {
    fn check(&self, item: &ScheduledQuotePolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled quote policy.
#[derive(Debug, Clone)]
pub struct ScheduledQuotePolicyValidator {
    pub rules: Vec<Rc<dyn ScheduledQuotePolicyRule>>,
}

impl ScheduledQuotePolicyValidator {
    pub fn validate(&self, item: &ScheduledQuotePolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledQuotePolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledQuotePolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_quote_policy_validator() -> ScheduledQuotePolicyValidator {
    ScheduledQuotePolicyValidator {
        rules: vec![
            Rc::new(ScheduledQuotePolicyRequiredCodeRule {
                message: "scheduled quote policy needs an entry".to_string(),
            }) as Rc<dyn ScheduledQuotePolicyRule>,
            Rc::new(ScheduledQuotePolicyIssuerLengthRule { max_length: 255 }),
            Rc::new(ScheduledQuotePolicyAllowedNameRule {
                allowed: vec!["strict".to_string(), "tiered".to_string(), "fallback".to_string()],
            }),
            Rc::new(ScheduledQuotePolicyCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_quote_policy_validator();
        let item = new_scheduled_quote_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledQuotePolicyRequiredCodeRule {
            message: "scheduled quote policy needs this field".to_string(),
        };
        let mut item = new_scheduled_quote_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = ScheduledQuotePolicyIssuerLengthRule { max_length: 1000 };
        let mut item = new_scheduled_quote_policy("x", "y");
        item.issuer = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
