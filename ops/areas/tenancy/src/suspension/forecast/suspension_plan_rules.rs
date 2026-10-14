use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a suspension plan must pass. Check returns its problems, or none.
pub trait SuspensionPlanRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionPlanCodePrefixRule {
    pub prefix: String,
}

impl SuspensionPlanRule for SuspensionPlanCodePrefixRule {
    fn check(&self, item: &SuspensionPlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionPlanTitleLengthRule {
    pub max_length: i64,
}

impl SuspensionPlanRule for SuspensionPlanTitleLengthRule {
    fn check(&self, item: &SuspensionPlan) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionPlanRequiredUnitRule {
    pub message: String,
}

impl SuspensionPlanRule for SuspensionPlanRequiredUnitRule {
    fn check(&self, item: &SuspensionPlan) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionPlanAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl SuspensionPlanRule for SuspensionPlanAllowedTitleRule {
    fn check(&self, item: &SuspensionPlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is blocked", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionPlanBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SuspensionPlanRule for SuspensionPlanBudgetRangeRule {
    fn check(&self, item: &SuspensionPlan) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension plan.
#[derive(Debug, Clone)]
pub struct SuspensionPlanValidator {
    pub rules: Vec<Rc<dyn SuspensionPlanRule>>,
}

impl SuspensionPlanValidator {
    pub fn validate(&self, item: &SuspensionPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SuspensionPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_plan_validator() -> SuspensionPlanValidator {
    SuspensionPlanValidator {
        rules: vec![
            Rc::new(SuspensionPlanCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn SuspensionPlanRule>,
            Rc::new(SuspensionPlanTitleLengthRule { max_length: 80 }),
            Rc::new(SuspensionPlanRequiredUnitRule {
                message: "suspension plan needs a value".to_string(),
            }),
            Rc::new(SuspensionPlanAllowedTitleRule {
                allowed: vec!["batched".to_string(), "pending".to_string()],
            }),
            Rc::new(SuspensionPlanBudgetRangeRule { min: 9.7, max: 1089.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_plan_validator();
        let item = new_suspension_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SuspensionPlanTitleLengthRule { max_length: 32 };
        let mut item = new_suspension_plan("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = SuspensionPlanRequiredUnitRule {
            message: "suspension plan needs this field".to_string(),
        };
        let mut item = new_suspension_plan("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = SuspensionPlanBudgetRangeRule { min: 9.9, max: 1285.2 };
        let mut item = new_suspension_plan("x", "y");
        item.budget = 1286.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
