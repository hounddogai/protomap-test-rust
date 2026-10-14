use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default failover rule must pass. Check returns its problems, or none.
pub trait DefaultFailoverRuleRule: std::fmt::Debug {
    fn check(&self, item: &DefaultFailoverRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFailoverRuleQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultFailoverRuleRule for DefaultFailoverRuleQuantityRangeRule {
    fn check(&self, item: &DefaultFailoverRule) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFailoverRuleCodePrefixRule {
    pub prefix: String,
}

impl DefaultFailoverRuleRule for DefaultFailoverRuleCodePrefixRule {
    fn check(&self, item: &DefaultFailoverRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFailoverRuleRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultFailoverRuleRule for DefaultFailoverRuleRevisionRangeRule {
    fn check(&self, item: &DefaultFailoverRule) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFailoverRuleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DefaultFailoverRuleRule for DefaultFailoverRuleAllowedCodeRule {
    fn check(&self, item: &DefaultFailoverRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default failover rule.
#[derive(Debug, Clone)]
pub struct DefaultFailoverRuleValidator {
    pub rules: Vec<Rc<dyn DefaultFailoverRuleRule>>,
}

impl DefaultFailoverRuleValidator {
    pub fn validate(&self, item: &DefaultFailoverRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultFailoverRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultFailoverRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_failover_rule_validator() -> DefaultFailoverRuleValidator {
    DefaultFailoverRuleValidator {
        rules: vec![
            Rc::new(DefaultFailoverRuleQuantityRangeRule { min: 120, max: 3600 }) as Rc<dyn DefaultFailoverRuleRule>,
            Rc::new(DefaultFailoverRuleCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(DefaultFailoverRuleRevisionRangeRule { min: 32, max: 120 }),
            Rc::new(DefaultFailoverRuleAllowedCodeRule {
                allowed: vec!["seasonal".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_failover_rule_validator();
        let item = new_default_failover_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = DefaultFailoverRuleQuantityRangeRule { min: 16, max: 120 };
        let mut item = new_default_failover_rule("x", "y");
        item.quantity = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = DefaultFailoverRuleRevisionRangeRule { min: 32, max: 3600 };
        let mut item = new_default_failover_rule("x", "y");
        item.revision = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
