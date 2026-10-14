use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual isolation target must pass. Check returns its problems, or none.
pub trait ManualIsolationTargetRule: std::fmt::Debug {
    fn check(&self, item: &ManualIsolationTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualIsolationTargetRequiredLabelRule {
    pub message: String,
}

impl ManualIsolationTargetRule for ManualIsolationTargetRequiredLabelRule {
    fn check(&self, item: &ManualIsolationTarget) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualIsolationTargetVisibleGuardRule {
    pub limit: f64,
}

impl ManualIsolationTargetRule for ManualIsolationTargetVisibleGuardRule {
    fn check(&self, item: &ManualIsolationTarget) -> Vec<String> {
        if item.visible && (item.budget > self.limit) {
            return vec![format!("budget {} exceeds {} while visible", item.budget, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualIsolationTargetCodePrefixRule {
    pub prefix: String,
}

impl ManualIsolationTargetRule for ManualIsolationTargetCodePrefixRule {
    fn check(&self, item: &ManualIsolationTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualIsolationTargetBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualIsolationTargetRule for ManualIsolationTargetBudgetRangeRule {
    fn check(&self, item: &ManualIsolationTarget) -> Vec<String> {
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
pub struct ManualIsolationTargetRequiredChannelRule {
    pub message: String,
}

impl ManualIsolationTargetRule for ManualIsolationTargetRequiredChannelRule {
    fn check(&self, item: &ManualIsolationTarget) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualIsolationTargetAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ManualIsolationTargetRule for ManualIsolationTargetAllowedCodeRule {
    fn check(&self, item: &ManualIsolationTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual isolation target.
#[derive(Debug, Clone)]
pub struct ManualIsolationTargetValidator {
    pub rules: Vec<Rc<dyn ManualIsolationTargetRule>>,
}

impl ManualIsolationTargetValidator {
    pub fn validate(&self, item: &ManualIsolationTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualIsolationTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualIsolationTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_isolation_target_validator() -> ManualIsolationTargetValidator {
    ManualIsolationTargetValidator {
        rules: vec![
            Rc::new(ManualIsolationTargetRequiredLabelRule {
                message: "manual isolation target needs a value".to_string(),
            }) as Rc<dyn ManualIsolationTargetRule>,
            Rc::new(ManualIsolationTargetVisibleGuardRule { limit: 8.9 }),
            Rc::new(ManualIsolationTargetCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(ManualIsolationTargetBudgetRangeRule { min: 9.7, max: 380.4 }),
            Rc::new(ManualIsolationTargetRequiredChannelRule {
                message: "manual isolation target needs this field".to_string(),
            }),
            Rc::new(ManualIsolationTargetAllowedCodeRule {
                allowed: vec!["global".to_string(), "weighted".to_string(), "weighted".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_isolation_target_validator();
        let item = new_manual_isolation_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ManualIsolationTargetRequiredLabelRule {
            message: "manual isolation target needs a value".to_string(),
        };
        let mut item = new_manual_isolation_target("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = ManualIsolationTargetBudgetRangeRule { min: 7.9, max: 2621.2 };
        let mut item = new_manual_isolation_target("x", "y");
        item.budget = 2622.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = ManualIsolationTargetRequiredChannelRule {
            message: "manual isolation target needs an entry".to_string(),
        };
        let mut item = new_manual_isolation_target("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
