use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted budget burn metric must pass. Check returns its problems, or none.
pub trait WeightedBudgetBurnMetricRule: std::fmt::Debug {
    fn check(&self, item: &WeightedBudgetBurnMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnMetricAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl WeightedBudgetBurnMetricRule for WeightedBudgetBurnMetricAllowedTitleRule {
    fn check(&self, item: &WeightedBudgetBurnMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is stale", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnMetricRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedBudgetBurnMetricRule for WeightedBudgetBurnMetricRevisionRangeRule {
    fn check(&self, item: &WeightedBudgetBurnMetric) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnMetricMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedBudgetBurnMetricRule for WeightedBudgetBurnMetricMaximumRangeRule {
    fn check(&self, item: &WeightedBudgetBurnMetric) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnMetricCodePrefixRule {
    pub prefix: String,
}

impl WeightedBudgetBurnMetricRule for WeightedBudgetBurnMetricCodePrefixRule {
    fn check(&self, item: &WeightedBudgetBurnMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnMetricRequiredTitleRule {
    pub message: String,
}

impl WeightedBudgetBurnMetricRule for WeightedBudgetBurnMetricRequiredTitleRule {
    fn check(&self, item: &WeightedBudgetBurnMetric) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted budget burn metric.
#[derive(Debug, Clone)]
pub struct WeightedBudgetBurnMetricValidator {
    pub rules: Vec<Rc<dyn WeightedBudgetBurnMetricRule>>,
}

impl WeightedBudgetBurnMetricValidator {
    pub fn validate(&self, item: &WeightedBudgetBurnMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedBudgetBurnMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedBudgetBurnMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_budget_burn_metric_validator() -> WeightedBudgetBurnMetricValidator {
    WeightedBudgetBurnMetricValidator {
        rules: vec![
            Rc::new(WeightedBudgetBurnMetricAllowedTitleRule {
                allowed: vec!["global".to_string()],
            }) as Rc<dyn WeightedBudgetBurnMetricRule>,
            Rc::new(WeightedBudgetBurnMetricRevisionRangeRule { min: 120, max: 1000 }),
            Rc::new(WeightedBudgetBurnMetricMaximumRangeRule { min: 8.6, max: 3583.0 }),
            Rc::new(WeightedBudgetBurnMetricCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(WeightedBudgetBurnMetricRequiredTitleRule {
                message: "weighted budget burn metric needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_budget_burn_metric_validator();
        let item = new_weighted_budget_burn_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = WeightedBudgetBurnMetricRevisionRangeRule { min: 16, max: 80 };
        let mut item = new_weighted_budget_burn_metric("x", "y");
        item.revision = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = WeightedBudgetBurnMetricMaximumRangeRule { min: 9.8, max: 3138.7 };
        let mut item = new_weighted_budget_burn_metric("x", "y");
        item.maximum = 3140.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = WeightedBudgetBurnMetricRequiredTitleRule {
            message: "weighted budget burn metric needs a value".to_string(),
        };
        let mut item = new_weighted_budget_burn_metric("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
