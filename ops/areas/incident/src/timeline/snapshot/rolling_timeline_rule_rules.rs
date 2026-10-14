use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling timeline rule must pass. Check returns its problems, or none.
pub trait RollingTimelineRuleRule: std::fmt::Debug {
    fn check(&self, item: &RollingTimelineRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTimelineRuleAllowedNamespaceRule {
    pub allowed: Vec<String>,
}

impl RollingTimelineRuleRule for RollingTimelineRuleAllowedNamespaceRule {
    fn check(&self, item: &RollingTimelineRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.namespace)))
        {
            return vec![format!("namespace {} is inconsistent", item.namespace)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTimelineRuleContactEmailLengthRule {
    pub max_length: i64,
}

impl RollingTimelineRuleRule for RollingTimelineRuleContactEmailLengthRule {
    fn check(&self, item: &RollingTimelineRule) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTimelineRuleMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingTimelineRuleRule for RollingTimelineRuleMaximumRangeRule {
    fn check(&self, item: &RollingTimelineRule) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling timeline rule.
#[derive(Debug, Clone)]
pub struct RollingTimelineRuleValidator {
    pub rules: Vec<Rc<dyn RollingTimelineRuleRule>>,
}

impl RollingTimelineRuleValidator {
    pub fn validate(&self, item: &RollingTimelineRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingTimelineRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingTimelineRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_timeline_rule_validator() -> RollingTimelineRuleValidator {
    RollingTimelineRuleValidator {
        rules: vec![
            Rc::new(RollingTimelineRuleAllowedNamespaceRule {
                allowed: vec!["lenient".to_string(), "manual".to_string()],
            }) as Rc<dyn RollingTimelineRuleRule>,
            Rc::new(RollingTimelineRuleContactEmailLengthRule { max_length: 255 }),
            Rc::new(RollingTimelineRuleMaximumRangeRule { min: 7.8, max: 846.6 }),
        ],
    }
}
