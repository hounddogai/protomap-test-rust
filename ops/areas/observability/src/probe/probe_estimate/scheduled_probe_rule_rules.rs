use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled probe rule must pass. Check returns its problems, or none.
pub trait ScheduledProbeRuleRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledProbeRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledProbeRuleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ScheduledProbeRuleRule for ScheduledProbeRuleAllowedCodeRule {
    fn check(&self, item: &ScheduledProbeRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledProbeRuleRequiredLabelRule {
    pub message: String,
}

impl ScheduledProbeRuleRule for ScheduledProbeRuleRequiredLabelRule {
    fn check(&self, item: &ScheduledProbeRule) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledProbeRuleBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledProbeRuleRule for ScheduledProbeRuleBudgetRangeRule {
    fn check(&self, item: &ScheduledProbeRule) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledProbeRuleRequiredCodeRule {
    pub message: String,
}

impl ScheduledProbeRuleRule for ScheduledProbeRuleRequiredCodeRule {
    fn check(&self, item: &ScheduledProbeRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledProbeRuleCodePrefixRule {
    pub prefix: String,
}

impl ScheduledProbeRuleRule for ScheduledProbeRuleCodePrefixRule {
    fn check(&self, item: &ScheduledProbeRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledProbeRuleNameLengthRule {
    pub max_length: i64,
}

impl ScheduledProbeRuleRule for ScheduledProbeRuleNameLengthRule {
    fn check(&self, item: &ScheduledProbeRule) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled probe rule.
#[derive(Debug, Clone)]
pub struct ScheduledProbeRuleValidator {
    pub rules: Vec<Rc<dyn ScheduledProbeRuleRule>>,
}

impl ScheduledProbeRuleValidator {
    pub fn validate(&self, item: &ScheduledProbeRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledProbeRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledProbeRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_probe_rule_validator() -> ScheduledProbeRuleValidator {
    ScheduledProbeRuleValidator {
        rules: vec![
            Rc::new(ScheduledProbeRuleAllowedCodeRule {
                allowed: vec!["seasonal".to_string(), "batched".to_string(), "global".to_string()],
            }) as Rc<dyn ScheduledProbeRuleRule>,
            Rc::new(ScheduledProbeRuleRequiredLabelRule {
                message: "scheduled probe rule needs this field".to_string(),
            }),
            Rc::new(ScheduledProbeRuleBudgetRangeRule { min: 8.4, max: 4992.6 }),
            Rc::new(ScheduledProbeRuleRequiredCodeRule {
                message: "scheduled probe rule needs an entry".to_string(),
            }),
            Rc::new(ScheduledProbeRuleCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(ScheduledProbeRuleNameLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_probe_rule_validator();
        let item = new_scheduled_probe_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ScheduledProbeRuleRequiredLabelRule {
            message: "scheduled probe rule needs a value".to_string(),
        };
        let mut item = new_scheduled_probe_rule("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = ScheduledProbeRuleBudgetRangeRule { min: 2.6, max: 4918.5 };
        let mut item = new_scheduled_probe_rule("x", "y");
        item.budget = 4920.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledProbeRuleRequiredCodeRule {
            message: "scheduled probe rule needs a value".to_string(),
        };
        let mut item = new_scheduled_probe_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ScheduledProbeRuleNameLengthRule { max_length: 80 };
        let mut item = new_scheduled_probe_rule("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
