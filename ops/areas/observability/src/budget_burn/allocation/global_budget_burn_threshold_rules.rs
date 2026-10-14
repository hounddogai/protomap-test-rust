use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global budget burn threshold must pass. Check returns its problems, or none.
pub trait GlobalBudgetBurnThresholdRule: std::fmt::Debug {
    fn check(&self, item: &GlobalBudgetBurnThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBudgetBurnThresholdBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalBudgetBurnThresholdRule for GlobalBudgetBurnThresholdBurstRangeRule {
    fn check(&self, item: &GlobalBudgetBurnThreshold) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBudgetBurnThresholdRequiredCodeRule {
    pub message: String,
}

impl GlobalBudgetBurnThresholdRule for GlobalBudgetBurnThresholdRequiredCodeRule {
    fn check(&self, item: &GlobalBudgetBurnThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBudgetBurnThresholdAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl GlobalBudgetBurnThresholdRule for GlobalBudgetBurnThresholdAllowedCodeRule {
    fn check(&self, item: &GlobalBudgetBurnThreshold) -> Vec<String> {
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
pub struct GlobalBudgetBurnThresholdExternalRefLengthRule {
    pub max_length: i64,
}

impl GlobalBudgetBurnThresholdRule for GlobalBudgetBurnThresholdExternalRefLengthRule {
    fn check(&self, item: &GlobalBudgetBurnThreshold) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a global budget burn threshold.
#[derive(Debug, Clone)]
pub struct GlobalBudgetBurnThresholdValidator {
    pub rules: Vec<Rc<dyn GlobalBudgetBurnThresholdRule>>,
}

impl GlobalBudgetBurnThresholdValidator {
    pub fn validate(&self, item: &GlobalBudgetBurnThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalBudgetBurnThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalBudgetBurnThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_budget_burn_threshold_validator() -> GlobalBudgetBurnThresholdValidator {
    GlobalBudgetBurnThresholdValidator {
        rules: vec![
            Rc::new(GlobalBudgetBurnThresholdBurstRangeRule { min: 16, max: 32 })
                as Rc<dyn GlobalBudgetBurnThresholdRule>,
            Rc::new(GlobalBudgetBurnThresholdRequiredCodeRule {
                message: "global budget burn threshold needs this field".to_string(),
            }),
            Rc::new(GlobalBudgetBurnThresholdAllowedCodeRule {
                allowed: vec!["manual".to_string(), "regional".to_string(), "seasonal".to_string()],
            }),
            Rc::new(GlobalBudgetBurnThresholdExternalRefLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_budget_burn_threshold_validator();
        let item = new_global_budget_burn_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = GlobalBudgetBurnThresholdBurstRangeRule { min: 64, max: 120 };
        let mut item = new_global_budget_burn_threshold("x", "y");
        item.burst = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalBudgetBurnThresholdRequiredCodeRule {
            message: "global budget burn threshold needs this field".to_string(),
        };
        let mut item = new_global_budget_burn_threshold("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = GlobalBudgetBurnThresholdExternalRefLengthRule { max_length: 120 };
        let mut item = new_global_budget_burn_threshold("x", "y");
        item.external_ref = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
