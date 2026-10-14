use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a budget burn plan must pass. Check returns its problems, or none.
pub trait BudgetBurnPlanRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnPlanPinnedGuardRule {
    pub limit: f64,
}

impl BudgetBurnPlanRule for BudgetBurnPlanPinnedGuardRule {
    fn check(&self, item: &BudgetBurnPlan) -> Vec<String> {
        if item.pinned && (item.threshold > self.limit) {
            return vec![format!(
                "threshold {} exceeds {} while pinned",
                item.threshold, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnPlanCodePrefixRule {
    pub prefix: String,
}

impl BudgetBurnPlanRule for BudgetBurnPlanCodePrefixRule {
    fn check(&self, item: &BudgetBurnPlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnPlanPhoneLengthRule {
    pub max_length: i64,
}

impl BudgetBurnPlanRule for BudgetBurnPlanPhoneLengthRule {
    fn check(&self, item: &BudgetBurnPlan) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnPlanThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BudgetBurnPlanRule for BudgetBurnPlanThresholdRangeRule {
    fn check(&self, item: &BudgetBurnPlan) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnPlanRequiredLabelRule {
    pub message: String,
}

impl BudgetBurnPlanRule for BudgetBurnPlanRequiredLabelRule {
    fn check(&self, item: &BudgetBurnPlan) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnPlanIssuerLengthRule {
    pub max_length: i64,
}

impl BudgetBurnPlanRule for BudgetBurnPlanIssuerLengthRule {
    fn check(&self, item: &BudgetBurnPlan) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a budget burn plan.
#[derive(Debug, Clone)]
pub struct BudgetBurnPlanValidator {
    pub rules: Vec<Rc<dyn BudgetBurnPlanRule>>,
}

impl BudgetBurnPlanValidator {
    pub fn validate(&self, item: &BudgetBurnPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BudgetBurnPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_plan_validator() -> BudgetBurnPlanValidator {
    BudgetBurnPlanValidator {
        rules: vec![
            Rc::new(BudgetBurnPlanPinnedGuardRule { limit: 1.6 }) as Rc<dyn BudgetBurnPlanRule>,
            Rc::new(BudgetBurnPlanCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(BudgetBurnPlanPhoneLengthRule { max_length: 3600 }),
            Rc::new(BudgetBurnPlanThresholdRangeRule { min: 8.2, max: 1269.4 }),
            Rc::new(BudgetBurnPlanRequiredLabelRule {
                message: "budget burn plan needs an entry".to_string(),
            }),
            Rc::new(BudgetBurnPlanIssuerLengthRule { max_length: 80 }),
        ],
    }
}
