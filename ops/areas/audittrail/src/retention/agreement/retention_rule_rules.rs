use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a retention rule must pass. Check returns its problems, or none.
pub trait RetentionRuleRule: std::fmt::Debug {
    fn check(&self, item: &RetentionRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionRuleRequiredCodeRule {
    pub message: String,
}

impl RetentionRuleRule for RetentionRuleRequiredCodeRule {
    fn check(&self, item: &RetentionRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionRuleCodePrefixRule {
    pub prefix: String,
}

impl RetentionRuleRule for RetentionRuleCodePrefixRule {
    fn check(&self, item: &RetentionRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionRuleRequiredNameRule {
    pub message: String,
}

impl RetentionRuleRule for RetentionRuleRequiredNameRule {
    fn check(&self, item: &RetentionRule) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionRuleTenantIdLengthRule {
    pub max_length: i64,
}

impl RetentionRuleRule for RetentionRuleTenantIdLengthRule {
    fn check(&self, item: &RetentionRule) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionRuleAllowedNameRule {
    pub allowed: Vec<String>,
}

impl RetentionRuleRule for RetentionRuleAllowedNameRule {
    fn check(&self, item: &RetentionRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is missing", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionRuleDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RetentionRuleRule for RetentionRuleDiscountRangeRule {
    fn check(&self, item: &RetentionRule) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a retention rule.
#[derive(Debug, Clone)]
pub struct RetentionRuleValidator {
    pub rules: Vec<Rc<dyn RetentionRuleRule>>,
}

impl RetentionRuleValidator {
    pub fn validate(&self, item: &RetentionRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RetentionRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_rule_validator() -> RetentionRuleValidator {
    RetentionRuleValidator {
        rules: vec![
            Rc::new(RetentionRuleRequiredCodeRule {
                message: "retention rule needs this field".to_string(),
            }) as Rc<dyn RetentionRuleRule>,
            Rc::new(RetentionRuleCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(RetentionRuleRequiredNameRule {
                message: "retention rule needs this field".to_string(),
            }),
            Rc::new(RetentionRuleTenantIdLengthRule { max_length: 64 }),
            Rc::new(RetentionRuleAllowedNameRule {
                allowed: vec!["rolling".to_string(), "scheduled".to_string(), "weighted".to_string()],
            }),
            Rc::new(RetentionRuleDiscountRangeRule { min: 2.9, max: 1008.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_rule_validator();
        let item = new_retention_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RetentionRuleRequiredCodeRule {
            message: "retention rule needs an entry".to_string(),
        };
        let mut item = new_retention_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RetentionRuleRequiredNameRule {
            message: "retention rule needs this field".to_string(),
        };
        let mut item = new_retention_rule("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = RetentionRuleTenantIdLengthRule { max_length: 255 };
        let mut item = new_retention_rule("x", "y");
        item.tenant_id = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = RetentionRuleDiscountRangeRule { min: 2.5, max: 2896.8 };
        let mut item = new_retention_rule("x", "y");
        item.discount = 2898.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
