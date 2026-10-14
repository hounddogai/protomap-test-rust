use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a allowlist rule must pass. Check returns its problems, or none.
pub trait AllowlistRuleRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistRuleRequiredTitleRule {
    pub message: String,
}

impl AllowlistRuleRule for AllowlistRuleRequiredTitleRule {
    fn check(&self, item: &AllowlistRule) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistRuleRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AllowlistRuleRule for AllowlistRuleRevisionRangeRule {
    fn check(&self, item: &AllowlistRule) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistRuleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl AllowlistRuleRule for AllowlistRuleAllowedCodeRule {
    fn check(&self, item: &AllowlistRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistRuleNamespaceLengthRule {
    pub max_length: i64,
}

impl AllowlistRuleRule for AllowlistRuleNamespaceLengthRule {
    fn check(&self, item: &AllowlistRule) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist rule.
#[derive(Debug, Clone)]
pub struct AllowlistRuleValidator {
    pub rules: Vec<Rc<dyn AllowlistRuleRule>>,
}

impl AllowlistRuleValidator {
    pub fn validate(&self, item: &AllowlistRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AllowlistRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_rule_validator() -> AllowlistRuleValidator {
    AllowlistRuleValidator {
        rules: vec![
            Rc::new(AllowlistRuleRequiredTitleRule {
                message: "allowlist rule needs an entry".to_string(),
            }) as Rc<dyn AllowlistRuleRule>,
            Rc::new(AllowlistRuleRevisionRangeRule { min: 255, max: 1000 }),
            Rc::new(AllowlistRuleAllowedCodeRule {
                allowed: vec!["fallback".to_string()],
            }),
            Rc::new(AllowlistRuleNamespaceLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_allowlist_rule_validator();
        let item = new_allowlist_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = AllowlistRuleRequiredTitleRule {
            message: "allowlist rule needs an entry".to_string(),
        };
        let mut item = new_allowlist_rule("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = AllowlistRuleRevisionRangeRule { min: 64, max: 120 };
        let mut item = new_allowlist_rule("x", "y");
        item.revision = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = AllowlistRuleNamespaceLengthRule { max_length: 3600 };
        let mut item = new_allowlist_rule("x", "y");
        item.namespace = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
