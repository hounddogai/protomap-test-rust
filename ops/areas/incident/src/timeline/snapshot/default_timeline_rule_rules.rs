use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default timeline rule must pass. Check returns its problems, or none.
pub trait DefaultTimelineRuleRule: std::fmt::Debug {
    fn check(&self, item: &DefaultTimelineRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTimelineRulePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultTimelineRuleRule for DefaultTimelineRulePortRangeRule {
    fn check(&self, item: &DefaultTimelineRule) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTimelineRuleCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultTimelineRuleRule for DefaultTimelineRuleCostRangeRule {
    fn check(&self, item: &DefaultTimelineRule) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTimelineRuleCodePrefixRule {
    pub prefix: String,
}

impl DefaultTimelineRuleRule for DefaultTimelineRuleCodePrefixRule {
    fn check(&self, item: &DefaultTimelineRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTimelineRuleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DefaultTimelineRuleRule for DefaultTimelineRuleAllowedCodeRule {
    fn check(&self, item: &DefaultTimelineRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default timeline rule.
#[derive(Debug, Clone)]
pub struct DefaultTimelineRuleValidator {
    pub rules: Vec<Rc<dyn DefaultTimelineRuleRule>>,
}

impl DefaultTimelineRuleValidator {
    pub fn validate(&self, item: &DefaultTimelineRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultTimelineRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultTimelineRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_timeline_rule_validator() -> DefaultTimelineRuleValidator {
    DefaultTimelineRuleValidator {
        rules: vec![
            Rc::new(DefaultTimelineRulePortRangeRule { min: 16, max: 86400 }) as Rc<dyn DefaultTimelineRuleRule>,
            Rc::new(DefaultTimelineRuleCostRangeRule { min: 2.1, max: 2148.4 }),
            Rc::new(DefaultTimelineRuleCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(DefaultTimelineRuleAllowedCodeRule {
                allowed: vec!["legacy".to_string(), "tiered".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_timeline_rule_validator();
        let item = new_default_timeline_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = DefaultTimelineRulePortRangeRule { min: 64, max: 120 };
        let mut item = new_default_timeline_rule("x", "y");
        item.port = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = DefaultTimelineRuleCostRangeRule { min: 5.3, max: 4699.8 };
        let mut item = new_default_timeline_rule("x", "y");
        item.cost = 4701.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
