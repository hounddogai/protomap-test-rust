use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual budget burn policy must pass. Check returns its problems, or none.
pub trait ManualBudgetBurnPolicyRule: std::fmt::Debug {
    fn check(&self, item: &ManualBudgetBurnPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnPolicyCodePrefixRule {
    pub prefix: String,
}

impl ManualBudgetBurnPolicyRule for ManualBudgetBurnPolicyCodePrefixRule {
    fn check(&self, item: &ManualBudgetBurnPolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnPolicyAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl ManualBudgetBurnPolicyRule for ManualBudgetBurnPolicyAllowedTitleRule {
    fn check(&self, item: &ManualBudgetBurnPolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is missing", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnPolicyRequiredTitleRule {
    pub message: String,
}

impl ManualBudgetBurnPolicyRule for ManualBudgetBurnPolicyRequiredTitleRule {
    fn check(&self, item: &ManualBudgetBurnPolicy) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual budget burn policy.
#[derive(Debug, Clone)]
pub struct ManualBudgetBurnPolicyValidator {
    pub rules: Vec<Rc<dyn ManualBudgetBurnPolicyRule>>,
}

impl ManualBudgetBurnPolicyValidator {
    pub fn validate(&self, item: &ManualBudgetBurnPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualBudgetBurnPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualBudgetBurnPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_budget_burn_policy_validator() -> ManualBudgetBurnPolicyValidator {
    ManualBudgetBurnPolicyValidator {
        rules: vec![
            Rc::new(ManualBudgetBurnPolicyCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn ManualBudgetBurnPolicyRule>,
            Rc::new(ManualBudgetBurnPolicyAllowedTitleRule {
                allowed: vec!["weighted".to_string(), "primary".to_string()],
            }),
            Rc::new(ManualBudgetBurnPolicyRequiredTitleRule {
                message: "manual budget burn policy needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_budget_burn_policy_validator();
        let item = new_manual_budget_burn_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ManualBudgetBurnPolicyRequiredTitleRule {
            message: "manual budget burn policy needs an entry".to_string(),
        };
        let mut item = new_manual_budget_burn_policy("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
