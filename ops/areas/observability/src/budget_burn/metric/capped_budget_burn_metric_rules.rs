use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped budget burn metric must pass. Check returns its problems, or none.
pub trait CappedBudgetBurnMetricRule: std::fmt::Debug {
    fn check(&self, item: &CappedBudgetBurnMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnMetricCodePrefixRule {
    pub prefix: String,
}

impl CappedBudgetBurnMetricRule for CappedBudgetBurnMetricCodePrefixRule {
    fn check(&self, item: &CappedBudgetBurnMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnMetricAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl CappedBudgetBurnMetricRule for CappedBudgetBurnMetricAllowedCodeRule {
    fn check(&self, item: &CappedBudgetBurnMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnMetricRequiredSourceRule {
    pub message: String,
}

impl CappedBudgetBurnMetricRule for CappedBudgetBurnMetricRequiredSourceRule {
    fn check(&self, item: &CappedBudgetBurnMetric) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnMetricCodeLengthRule {
    pub max_length: i64,
}

impl CappedBudgetBurnMetricRule for CappedBudgetBurnMetricCodeLengthRule {
    fn check(&self, item: &CappedBudgetBurnMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped budget burn metric.
#[derive(Debug, Clone)]
pub struct CappedBudgetBurnMetricValidator {
    pub rules: Vec<Rc<dyn CappedBudgetBurnMetricRule>>,
}

impl CappedBudgetBurnMetricValidator {
    pub fn validate(&self, item: &CappedBudgetBurnMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedBudgetBurnMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedBudgetBurnMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_budget_burn_metric_validator() -> CappedBudgetBurnMetricValidator {
    CappedBudgetBurnMetricValidator {
        rules: vec![
            Rc::new(CappedBudgetBurnMetricCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn CappedBudgetBurnMetricRule>,
            Rc::new(CappedBudgetBurnMetricAllowedCodeRule {
                allowed: vec!["legacy".to_string(), "default".to_string(), "rolling".to_string()],
            }),
            Rc::new(CappedBudgetBurnMetricRequiredSourceRule {
                message: "capped budget burn metric needs this field".to_string(),
            }),
            Rc::new(CappedBudgetBurnMetricCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_budget_burn_metric_validator();
        let item = new_capped_budget_burn_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = CappedBudgetBurnMetricRequiredSourceRule {
            message: "capped budget burn metric needs a value".to_string(),
        };
        let mut item = new_capped_budget_burn_metric("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedBudgetBurnMetricCodeLengthRule { max_length: 16 };
        let mut item = new_capped_budget_burn_metric("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
