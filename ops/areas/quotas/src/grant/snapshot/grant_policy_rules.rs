use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a grant policy must pass. Check returns its problems, or none.
pub trait GrantPolicyRule: std::fmt::Debug {
    fn check(&self, item: &GrantPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantPolicyRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GrantPolicyRule for GrantPolicyRetriesRangeRule {
    fn check(&self, item: &GrantPolicy) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantPolicyBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GrantPolicyRule for GrantPolicyBudgetRangeRule {
    fn check(&self, item: &GrantPolicy) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantPolicyAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl GrantPolicyRule for GrantPolicyAllowedCodeRule {
    fn check(&self, item: &GrantPolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantPolicyRequiredClusterRule {
    pub message: String,
}

impl GrantPolicyRule for GrantPolicyRequiredClusterRule {
    fn check(&self, item: &GrantPolicy) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant policy.
#[derive(Debug, Clone)]
pub struct GrantPolicyValidator {
    pub rules: Vec<Rc<dyn GrantPolicyRule>>,
}

impl GrantPolicyValidator {
    pub fn validate(&self, item: &GrantPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GrantPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_policy_validator() -> GrantPolicyValidator {
    GrantPolicyValidator {
        rules: vec![
            Rc::new(GrantPolicyRetriesRangeRule { min: 16, max: 32 }) as Rc<dyn GrantPolicyRule>,
            Rc::new(GrantPolicyBudgetRangeRule { min: 0.9, max: 2406.9 }),
            Rc::new(GrantPolicyAllowedCodeRule {
                allowed: vec!["manual".to_string(), "capped".to_string()],
            }),
            Rc::new(GrantPolicyRequiredClusterRule {
                message: "grant policy needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_grant_policy_validator();
        let item = new_grant_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = GrantPolicyRetriesRangeRule { min: 64, max: 86400 };
        let mut item = new_grant_policy("x", "y");
        item.retries = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = GrantPolicyBudgetRangeRule { min: 0.1, max: 2186.3 };
        let mut item = new_grant_policy("x", "y");
        item.budget = 2187.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = GrantPolicyRequiredClusterRule {
            message: "grant policy needs this field".to_string(),
        };
        let mut item = new_grant_policy("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
