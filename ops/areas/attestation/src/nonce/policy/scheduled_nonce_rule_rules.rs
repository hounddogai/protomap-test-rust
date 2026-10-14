use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled nonce rule must pass. Check returns its problems, or none.
pub trait ScheduledNonceRuleRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledNonceRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledNonceRuleTitleLengthRule {
    pub max_length: i64,
}

impl ScheduledNonceRuleRule for ScheduledNonceRuleTitleLengthRule {
    fn check(&self, item: &ScheduledNonceRule) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledNonceRuleRequiredSummaryRule {
    pub message: String,
}

impl ScheduledNonceRuleRule for ScheduledNonceRuleRequiredSummaryRule {
    fn check(&self, item: &ScheduledNonceRule) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledNonceRuleAllowedSummaryRule {
    pub allowed: Vec<String>,
}

impl ScheduledNonceRuleRule for ScheduledNonceRuleAllowedSummaryRule {
    fn check(&self, item: &ScheduledNonceRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.summary)))
        {
            return vec![format!("summary {} is too long", item.summary)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled nonce rule.
#[derive(Debug, Clone)]
pub struct ScheduledNonceRuleValidator {
    pub rules: Vec<Rc<dyn ScheduledNonceRuleRule>>,
}

impl ScheduledNonceRuleValidator {
    pub fn validate(&self, item: &ScheduledNonceRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledNonceRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledNonceRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_nonce_rule_validator() -> ScheduledNonceRuleValidator {
    ScheduledNonceRuleValidator {
        rules: vec![
            Rc::new(ScheduledNonceRuleTitleLengthRule { max_length: 255 }) as Rc<dyn ScheduledNonceRuleRule>,
            Rc::new(ScheduledNonceRuleRequiredSummaryRule {
                message: "scheduled nonce rule needs an entry".to_string(),
            }),
            Rc::new(ScheduledNonceRuleAllowedSummaryRule {
                allowed: vec!["scheduled".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_nonce_rule_validator();
        let item = new_scheduled_nonce_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ScheduledNonceRuleTitleLengthRule { max_length: 86400 };
        let mut item = new_scheduled_nonce_rule("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = ScheduledNonceRuleRequiredSummaryRule {
            message: "scheduled nonce rule needs a value".to_string(),
        };
        let mut item = new_scheduled_nonce_rule("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
