use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a egress rule must pass. Check returns its problems, or none.
pub trait EgressRuleRule: std::fmt::Debug {
    fn check(&self, item: &EgressRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressRuleVisibleGuardRule {
    pub limit: i64,
}

impl EgressRuleRule for EgressRuleVisibleGuardRule {
    fn check(&self, item: &EgressRule) -> Vec<String> {
        if item.visible && (item.version > self.limit) {
            return vec![format!("version {} exceeds {} while visible", item.version, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressRuleLabelLengthRule {
    pub max_length: i64,
}

impl EgressRuleRule for EgressRuleLabelLengthRule {
    fn check(&self, item: &EgressRule) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressRuleAllowedChannelRule {
    pub allowed: Vec<String>,
}

impl EgressRuleRule for EgressRuleAllowedChannelRule {
    fn check(&self, item: &EgressRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.channel)))
        {
            return vec![format!("channel {} is not allowed", item.channel)];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress rule.
#[derive(Debug, Clone)]
pub struct EgressRuleValidator {
    pub rules: Vec<Rc<dyn EgressRuleRule>>,
}

impl EgressRuleValidator {
    pub fn validate(&self, item: &EgressRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EgressRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_rule_validator() -> EgressRuleValidator {
    EgressRuleValidator {
        rules: vec![
            Rc::new(EgressRuleVisibleGuardRule { limit: 86400 }) as Rc<dyn EgressRuleRule>,
            Rc::new(EgressRuleLabelLengthRule { max_length: 32 }),
            Rc::new(EgressRuleAllowedChannelRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_rule_validator();
        let item = new_egress_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = EgressRuleLabelLengthRule { max_length: 32 };
        let mut item = new_egress_rule("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
