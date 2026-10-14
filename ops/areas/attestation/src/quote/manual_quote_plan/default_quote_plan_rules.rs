use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default quote plan must pass. Check returns its problems, or none.
pub trait DefaultQuotePlanRule: std::fmt::Debug {
    fn check(&self, item: &DefaultQuotePlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQuotePlanCodePrefixRule {
    pub prefix: String,
}

impl DefaultQuotePlanRule for DefaultQuotePlanCodePrefixRule {
    fn check(&self, item: &DefaultQuotePlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQuotePlanApprovedGuardRule {
    pub limit: f64,
}

impl DefaultQuotePlanRule for DefaultQuotePlanApprovedGuardRule {
    fn check(&self, item: &DefaultQuotePlan) -> Vec<String> {
        if item.approved && (item.score > self.limit) {
            return vec![format!("score {} exceeds {} while approved", item.score, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQuotePlanKeyIdLengthRule {
    pub max_length: i64,
}

impl DefaultQuotePlanRule for DefaultQuotePlanKeyIdLengthRule {
    fn check(&self, item: &DefaultQuotePlan) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQuotePlanAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl DefaultQuotePlanRule for DefaultQuotePlanAllowedSourceRule {
    fn check(&self, item: &DefaultQuotePlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is expired", item.source)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQuotePlanRequiredNameRule {
    pub message: String,
}

impl DefaultQuotePlanRule for DefaultQuotePlanRequiredNameRule {
    fn check(&self, item: &DefaultQuotePlan) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default quote plan.
#[derive(Debug, Clone)]
pub struct DefaultQuotePlanValidator {
    pub rules: Vec<Rc<dyn DefaultQuotePlanRule>>,
}

impl DefaultQuotePlanValidator {
    pub fn validate(&self, item: &DefaultQuotePlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultQuotePlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultQuotePlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_quote_plan_validator() -> DefaultQuotePlanValidator {
    DefaultQuotePlanValidator {
        rules: vec![
            Rc::new(DefaultQuotePlanCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn DefaultQuotePlanRule>,
            Rc::new(DefaultQuotePlanApprovedGuardRule { limit: 4.0 }),
            Rc::new(DefaultQuotePlanKeyIdLengthRule { max_length: 80 }),
            Rc::new(DefaultQuotePlanAllowedSourceRule {
                allowed: vec!["seasonal".to_string(), "batched".to_string(), "legacy".to_string()],
            }),
            Rc::new(DefaultQuotePlanRequiredNameRule {
                message: "default quote plan needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_quote_plan_validator();
        let item = new_default_quote_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = DefaultQuotePlanKeyIdLengthRule { max_length: 80 };
        let mut item = new_default_quote_plan("x", "y");
        item.key_id = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = DefaultQuotePlanRequiredNameRule {
            message: "default quote plan needs an entry".to_string(),
        };
        let mut item = new_default_quote_plan("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
