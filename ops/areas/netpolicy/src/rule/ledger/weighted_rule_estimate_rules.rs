use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted rule estimate must pass. Check returns its problems, or none.
pub trait WeightedRuleEstimateRule: std::fmt::Debug {
    fn check(&self, item: &WeightedRuleEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleEstimateMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedRuleEstimateRule for WeightedRuleEstimateMarginRangeRule {
    fn check(&self, item: &WeightedRuleEstimate) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleEstimateAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl WeightedRuleEstimateRule for WeightedRuleEstimateAllowedTitleRule {
    fn check(&self, item: &WeightedRuleEstimate) -> Vec<String> {
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
pub struct WeightedRuleEstimateRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedRuleEstimateRule for WeightedRuleEstimateRateRangeRule {
    fn check(&self, item: &WeightedRuleEstimate) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleEstimateActiveGuardRule {
    pub limit: i64,
}

impl WeightedRuleEstimateRule for WeightedRuleEstimateActiveGuardRule {
    fn check(&self, item: &WeightedRuleEstimate) -> Vec<String> {
        if item.active && (item.quota_units > self.limit) {
            return vec![format!(
                "quota units {} exceeds {} while active",
                item.quota_units, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleEstimateTitleLengthRule {
    pub max_length: i64,
}

impl WeightedRuleEstimateRule for WeightedRuleEstimateTitleLengthRule {
    fn check(&self, item: &WeightedRuleEstimate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted rule estimate.
#[derive(Debug, Clone)]
pub struct WeightedRuleEstimateValidator {
    pub rules: Vec<Rc<dyn WeightedRuleEstimateRule>>,
}

impl WeightedRuleEstimateValidator {
    pub fn validate(&self, item: &WeightedRuleEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedRuleEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedRuleEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_rule_estimate_validator() -> WeightedRuleEstimateValidator {
    WeightedRuleEstimateValidator {
        rules: vec![
            Rc::new(WeightedRuleEstimateMarginRangeRule { min: 9.2, max: 1478.7 }) as Rc<dyn WeightedRuleEstimateRule>,
            Rc::new(WeightedRuleEstimateAllowedTitleRule {
                allowed: vec!["regional".to_string(), "pending".to_string()],
            }),
            Rc::new(WeightedRuleEstimateRateRangeRule { min: 2.9, max: 933.2 }),
            Rc::new(WeightedRuleEstimateActiveGuardRule { limit: 3600 }),
            Rc::new(WeightedRuleEstimateTitleLengthRule { max_length: 16 }),
        ],
    }
}
