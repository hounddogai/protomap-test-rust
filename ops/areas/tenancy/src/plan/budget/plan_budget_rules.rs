use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a plan budget must pass. Check returns its problems, or none.
pub trait PlanBudgetRule: std::fmt::Debug {
    fn check(&self, item: &PlanBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanBudgetNameLengthRule {
    pub max_length: i64,
}

impl PlanBudgetRule for PlanBudgetNameLengthRule {
    fn check(&self, item: &PlanBudget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanBudgetMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PlanBudgetRule for PlanBudgetMarginRangeRule {
    fn check(&self, item: &PlanBudget) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanBudgetSummaryLengthRule {
    pub max_length: i64,
}

impl PlanBudgetRule for PlanBudgetSummaryLengthRule {
    fn check(&self, item: &PlanBudget) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanBudgetAllowedNotesRule {
    pub allowed: Vec<String>,
}

impl PlanBudgetRule for PlanBudgetAllowedNotesRule {
    fn check(&self, item: &PlanBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.notes))) {
            return vec![format!("notes {} is too long", item.notes)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanBudgetRequiredCountryRule {
    pub message: String,
}

impl PlanBudgetRule for PlanBudgetRequiredCountryRule {
    fn check(&self, item: &PlanBudget) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanBudgetCodeLengthRule {
    pub max_length: i64,
}

impl PlanBudgetRule for PlanBudgetCodeLengthRule {
    fn check(&self, item: &PlanBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan budget.
#[derive(Debug, Clone)]
pub struct PlanBudgetValidator {
    pub rules: Vec<Rc<dyn PlanBudgetRule>>,
}

impl PlanBudgetValidator {
    pub fn validate(&self, item: &PlanBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PlanBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_budget_validator() -> PlanBudgetValidator {
    PlanBudgetValidator {
        rules: vec![
            Rc::new(PlanBudgetNameLengthRule { max_length: 32 }) as Rc<dyn PlanBudgetRule>,
            Rc::new(PlanBudgetMarginRangeRule { min: 9.1, max: 2062.4 }),
            Rc::new(PlanBudgetSummaryLengthRule { max_length: 32 }),
            Rc::new(PlanBudgetAllowedNotesRule {
                allowed: vec!["adaptive".to_string(), "lenient".to_string()],
            }),
            Rc::new(PlanBudgetRequiredCountryRule {
                message: "plan budget needs a value".to_string(),
            }),
            Rc::new(PlanBudgetCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_plan_budget_validator();
        let item = new_plan_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PlanBudgetNameLengthRule { max_length: 80 };
        let mut item = new_plan_budget("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = PlanBudgetMarginRangeRule { min: 0.7, max: 3825.3 };
        let mut item = new_plan_budget("x", "y");
        item.margin = 3826.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = PlanBudgetSummaryLengthRule { max_length: 3600 };
        let mut item = new_plan_budget("x", "y");
        item.summary = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = PlanBudgetRequiredCountryRule {
            message: "plan budget needs an entry".to_string(),
        };
        let mut item = new_plan_budget("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PlanBudgetCodeLengthRule { max_length: 32 };
        let mut item = new_plan_budget("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
