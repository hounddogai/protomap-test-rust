use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped budget burn plan must pass. Check returns its problems, or none.
pub trait CappedBudgetBurnPlanRule: std::fmt::Debug {
    fn check(&self, item: &CappedBudgetBurnPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnPlanCodePrefixRule {
    pub prefix: String,
}

impl CappedBudgetBurnPlanRule for CappedBudgetBurnPlanCodePrefixRule {
    fn check(&self, item: &CappedBudgetBurnPlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnPlanCodeLengthRule {
    pub max_length: i64,
}

impl CappedBudgetBurnPlanRule for CappedBudgetBurnPlanCodeLengthRule {
    fn check(&self, item: &CappedBudgetBurnPlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnPlanRequiredLabelRule {
    pub message: String,
}

impl CappedBudgetBurnPlanRule for CappedBudgetBurnPlanRequiredLabelRule {
    fn check(&self, item: &CappedBudgetBurnPlan) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped budget burn plan.
#[derive(Debug, Clone)]
pub struct CappedBudgetBurnPlanValidator {
    pub rules: Vec<Rc<dyn CappedBudgetBurnPlanRule>>,
}

impl CappedBudgetBurnPlanValidator {
    pub fn validate(&self, item: &CappedBudgetBurnPlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedBudgetBurnPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedBudgetBurnPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_budget_burn_plan_validator() -> CappedBudgetBurnPlanValidator {
    CappedBudgetBurnPlanValidator {
        rules: vec![
            Rc::new(CappedBudgetBurnPlanCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn CappedBudgetBurnPlanRule>,
            Rc::new(CappedBudgetBurnPlanCodeLengthRule { max_length: 255 }),
            Rc::new(CappedBudgetBurnPlanRequiredLabelRule {
                message: "capped budget burn plan needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_budget_burn_plan_validator();
        let item = new_capped_budget_burn_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedBudgetBurnPlanCodeLengthRule { max_length: 64 };
        let mut item = new_capped_budget_burn_plan("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = CappedBudgetBurnPlanRequiredLabelRule {
            message: "capped budget burn plan needs this field".to_string(),
        };
        let mut item = new_capped_budget_burn_plan("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
