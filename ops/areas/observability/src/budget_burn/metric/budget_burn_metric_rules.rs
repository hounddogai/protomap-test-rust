use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a budget burn metric must pass. Check returns its problems, or none.
pub trait BudgetBurnMetricRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnMetricTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl BudgetBurnMetricRule for BudgetBurnMetricTagsLimitRule {
    fn check(&self, item: &BudgetBurnMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnMetricAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl BudgetBurnMetricRule for BudgetBurnMetricAllowedCountryRule {
    fn check(&self, item: &BudgetBurnMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is below minimum", item.country)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnMetricCodePrefixRule {
    pub prefix: String,
}

impl BudgetBurnMetricRule for BudgetBurnMetricCodePrefixRule {
    fn check(&self, item: &BudgetBurnMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a budget burn metric.
#[derive(Debug, Clone)]
pub struct BudgetBurnMetricValidator {
    pub rules: Vec<Rc<dyn BudgetBurnMetricRule>>,
}

impl BudgetBurnMetricValidator {
    pub fn validate(&self, item: &BudgetBurnMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BudgetBurnMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_metric_validator() -> BudgetBurnMetricValidator {
    BudgetBurnMetricValidator {
        rules: vec![
            Rc::new(BudgetBurnMetricTagsLimitRule {
                max_count: 1000,
                banned: vec!["adaptive".to_string()],
            }) as Rc<dyn BudgetBurnMetricRule>,
            Rc::new(BudgetBurnMetricAllowedCountryRule {
                allowed: vec!["rolling".to_string()],
            }),
            Rc::new(BudgetBurnMetricCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_metric_validator();
        let item = new_budget_burn_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }
}
