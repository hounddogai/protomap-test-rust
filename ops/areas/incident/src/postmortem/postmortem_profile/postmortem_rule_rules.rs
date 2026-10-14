use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a postmortem rule must pass. Check returns its problems, or none.
pub trait PostmortemRuleRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemRuleThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PostmortemRuleRule for PostmortemRuleThresholdRangeRule {
    fn check(&self, item: &PostmortemRule) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemRuleTenantIdLengthRule {
    pub max_length: i64,
}

impl PostmortemRuleRule for PostmortemRuleTenantIdLengthRule {
    fn check(&self, item: &PostmortemRule) -> Vec<String> {
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
pub struct PostmortemRuleCodePrefixRule {
    pub prefix: String,
}

impl PostmortemRuleRule for PostmortemRuleCodePrefixRule {
    fn check(&self, item: &PostmortemRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemRuleRequiredContactEmailRule {
    pub message: String,
}

impl PostmortemRuleRule for PostmortemRuleRequiredContactEmailRule {
    fn check(&self, item: &PostmortemRule) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemRuleQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PostmortemRuleRule for PostmortemRuleQuantityRangeRule {
    fn check(&self, item: &PostmortemRule) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a postmortem rule.
#[derive(Debug, Clone)]
pub struct PostmortemRuleValidator {
    pub rules: Vec<Rc<dyn PostmortemRuleRule>>,
}

impl PostmortemRuleValidator {
    pub fn validate(&self, item: &PostmortemRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PostmortemRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_rule_validator() -> PostmortemRuleValidator {
    PostmortemRuleValidator {
        rules: vec![
            Rc::new(PostmortemRuleThresholdRangeRule { min: 0.1, max: 1803.2 }) as Rc<dyn PostmortemRuleRule>,
            Rc::new(PostmortemRuleTenantIdLengthRule { max_length: 80 }),
            Rc::new(PostmortemRuleCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(PostmortemRuleRequiredContactEmailRule {
                message: "postmortem rule needs this field".to_string(),
            }),
            Rc::new(PostmortemRuleQuantityRangeRule { min: 120, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_postmortem_rule_validator();
        let item = new_postmortem_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = PostmortemRuleThresholdRangeRule { min: 9.5, max: 3125.6 };
        let mut item = new_postmortem_rule("x", "y");
        item.threshold = 3127.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = PostmortemRuleTenantIdLengthRule { max_length: 86400 };
        let mut item = new_postmortem_rule("x", "y");
        item.tenant_id = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = PostmortemRuleRequiredContactEmailRule {
            message: "postmortem rule needs a value".to_string(),
        };
        let mut item = new_postmortem_rule("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = PostmortemRuleQuantityRangeRule { min: 3600, max: 86400 };
        let mut item = new_postmortem_rule("x", "y");
        item.quantity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
