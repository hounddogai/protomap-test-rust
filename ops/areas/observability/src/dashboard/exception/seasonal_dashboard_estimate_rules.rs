use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal dashboard estimate must pass. Check returns its problems, or none.
pub trait SeasonalDashboardEstimateRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalDashboardEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDashboardEstimateRegionLengthRule {
    pub max_length: i64,
}

impl SeasonalDashboardEstimateRule for SeasonalDashboardEstimateRegionLengthRule {
    fn check(&self, item: &SeasonalDashboardEstimate) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDashboardEstimateCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalDashboardEstimateRule for SeasonalDashboardEstimateCodeLengthRule {
    fn check(&self, item: &SeasonalDashboardEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDashboardEstimateCodePrefixRule {
    pub prefix: String,
}

impl SeasonalDashboardEstimateRule for SeasonalDashboardEstimateCodePrefixRule {
    fn check(&self, item: &SeasonalDashboardEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDashboardEstimateRequiredRegionRule {
    pub message: String,
}

impl SeasonalDashboardEstimateRule for SeasonalDashboardEstimateRequiredRegionRule {
    fn check(&self, item: &SeasonalDashboardEstimate) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDashboardEstimateBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalDashboardEstimateRule for SeasonalDashboardEstimateBudgetRangeRule {
    fn check(&self, item: &SeasonalDashboardEstimate) -> Vec<String> {
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
pub struct SeasonalDashboardEstimateCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalDashboardEstimateRule for SeasonalDashboardEstimateCeilingRangeRule {
    fn check(&self, item: &SeasonalDashboardEstimate) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal dashboard estimate.
#[derive(Debug, Clone)]
pub struct SeasonalDashboardEstimateValidator {
    pub rules: Vec<Rc<dyn SeasonalDashboardEstimateRule>>,
}

impl SeasonalDashboardEstimateValidator {
    pub fn validate(&self, item: &SeasonalDashboardEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalDashboardEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalDashboardEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_dashboard_estimate_validator() -> SeasonalDashboardEstimateValidator {
    SeasonalDashboardEstimateValidator {
        rules: vec![
            Rc::new(SeasonalDashboardEstimateRegionLengthRule { max_length: 64 })
                as Rc<dyn SeasonalDashboardEstimateRule>,
            Rc::new(SeasonalDashboardEstimateCodeLengthRule { max_length: 32 }),
            Rc::new(SeasonalDashboardEstimateCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(SeasonalDashboardEstimateRequiredRegionRule {
                message: "seasonal dashboard estimate needs a value".to_string(),
            }),
            Rc::new(SeasonalDashboardEstimateBudgetRangeRule { min: 0.0, max: 3068.7 }),
            Rc::new(SeasonalDashboardEstimateCeilingRangeRule { min: 2.3, max: 1337.4 }),
        ],
    }
}
