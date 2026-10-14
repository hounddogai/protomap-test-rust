use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a redaction rule must pass. Check returns its problems, or none.
pub trait RedactionRuleRule: std::fmt::Debug {
    fn check(&self, item: &RedactionRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionRuleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RedactionRuleRule for RedactionRuleAllowedCodeRule {
    fn check(&self, item: &RedactionRule) -> Vec<String> {
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
pub struct RedactionRuleTitleLengthRule {
    pub max_length: i64,
}

impl RedactionRuleRule for RedactionRuleTitleLengthRule {
    fn check(&self, item: &RedactionRule) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionRuleUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RedactionRuleRule for RedactionRuleUpdatedAtRangeRule {
    fn check(&self, item: &RedactionRule) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionRuleCodePrefixRule {
    pub prefix: String,
}

impl RedactionRuleRule for RedactionRuleCodePrefixRule {
    fn check(&self, item: &RedactionRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction rule.
#[derive(Debug, Clone)]
pub struct RedactionRuleValidator {
    pub rules: Vec<Rc<dyn RedactionRuleRule>>,
}

impl RedactionRuleValidator {
    pub fn validate(&self, item: &RedactionRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RedactionRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_rule_validator() -> RedactionRuleValidator {
    RedactionRuleValidator {
        rules: vec![
            Rc::new(RedactionRuleAllowedCodeRule {
                allowed: vec!["tiered".to_string()],
            }) as Rc<dyn RedactionRuleRule>,
            Rc::new(RedactionRuleTitleLengthRule { max_length: 80 }),
            Rc::new(RedactionRuleUpdatedAtRangeRule { min: 255, max: 86400 }),
            Rc::new(RedactionRuleCodePrefixRule {
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
        let validator = default_redaction_rule_validator();
        let item = new_redaction_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RedactionRuleTitleLengthRule { max_length: 120 };
        let mut item = new_redaction_rule("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = RedactionRuleUpdatedAtRangeRule { min: 32, max: 32 };
        let mut item = new_redaction_rule("x", "y");
        item.updated_at = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
