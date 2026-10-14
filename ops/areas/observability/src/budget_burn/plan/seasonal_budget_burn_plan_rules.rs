use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal budget burn plan must pass. Check returns its problems, or none.
pub trait SeasonalBudgetBurnPlanRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalBudgetBurnPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalBudgetBurnPlanVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalBudgetBurnPlanRule for SeasonalBudgetBurnPlanVersionRangeRule {
    fn check(&self, item: &SeasonalBudgetBurnPlan) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalBudgetBurnPlanRequiredCurrencyRule {
    pub message: String,
}

impl SeasonalBudgetBurnPlanRule for SeasonalBudgetBurnPlanRequiredCurrencyRule {
    fn check(&self, item: &SeasonalBudgetBurnPlan) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalBudgetBurnPlanRequiredFingerprintRule {
    pub message: String,
}

impl SeasonalBudgetBurnPlanRule for SeasonalBudgetBurnPlanRequiredFingerprintRule {
    fn check(&self, item: &SeasonalBudgetBurnPlan) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal budget burn plan.
#[derive(Debug, Clone)]
pub struct SeasonalBudgetBurnPlanValidator {
    pub rules: Vec<Rc<dyn SeasonalBudgetBurnPlanRule>>,
}

impl SeasonalBudgetBurnPlanValidator {
    pub fn validate(&self, item: &SeasonalBudgetBurnPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalBudgetBurnPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalBudgetBurnPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_budget_burn_plan_validator() -> SeasonalBudgetBurnPlanValidator {
    SeasonalBudgetBurnPlanValidator {
        rules: vec![
            Rc::new(SeasonalBudgetBurnPlanVersionRangeRule { min: 80, max: 3600 })
                as Rc<dyn SeasonalBudgetBurnPlanRule>,
            Rc::new(SeasonalBudgetBurnPlanRequiredCurrencyRule {
                message: "seasonal budget burn plan needs a value".to_string(),
            }),
            Rc::new(SeasonalBudgetBurnPlanRequiredFingerprintRule {
                message: "seasonal budget burn plan needs a value".to_string(),
            }),
        ],
    }
}
